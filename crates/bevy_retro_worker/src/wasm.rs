use std::{collections::HashMap, mem, ptr};

use async_channel::Sender;
use js_sys::{Array, ArrayBuffer, Uint8Array};
use lazy_static::lazy_static;
use std::sync::Mutex;
use uuid::Uuid;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, Worker};

lazy_static! {
    static ref TASK_POOL: BlockingTaskPool = BlockingTaskPool::create();
}

pub struct BlockingTaskPool {
    _worker_callback: Closure<dyn FnMut(MessageEvent)>,
    job_result_senders: Mutex<HashMap<Uuid, Sender<Vec<u8>>>>,
    worker: Mutex<Worker>,
}

// Correct me if I'm wrong, but it should be safe to implement sync for this because we only use it
// in single-threadded wasm. That said, it isn't really `Sync` so is this a good idea?
unsafe impl Sync for BlockingTaskPool {}

/// Trait that adds funtion to convert any type to the raw bytes of its memory representation
pub trait AsMemBytes {
    /// Get a reference to the type's raw memory representation
    unsafe fn as_mem_bytes(&self) -> &[u8];

    /// Get a [`Uint8Array`] copied from the type's raw memory representation
    unsafe fn copy_mem_bytes_to_new_arraybuffer(&self) -> Uint8Array;
}

impl<T> AsMemBytes for T {
    unsafe fn as_mem_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts(self as *const T as *const u8, std::mem::size_of::<T>())
    }

    unsafe fn copy_mem_bytes_to_new_arraybuffer(&self) -> Uint8Array {
        // Get a slice of the raw memory bytes
        let data_bytes = self.as_mem_bytes();
        // Create a new buffer of the size needed to hold the type
        let data_bytes_buffer = js_sys::Uint8Array::new_with_length(data_bytes.len() as u32);
        // Copy the bytes into the buffer
        data_bytes_buffer.copy_from(data_bytes);
        // Return the buffer
        data_bytes_buffer
    }
}

impl BlockingTaskPool {
    /// Forces the initialization of the worker task pool
    ///
    /// Even if this is not called the worker will be created if is an attempt later to spawn jobs
    /// on the pool.
    pub fn init() {
        &*TASK_POOL;
    }

    /// Creates the task pool
    fn create() -> Self {
        // Get the path to the web worker JavaScript
        let worker = web_sys::Worker::new(include_str!(concat!(
            env!("OUT_DIR"),
            "/web_worker_uri.txt"
        )))
        .unwrap();

        // Create the callback that will be run when we get messages from our worker
        let worker_callback = Closure::wrap(Box::new(|event: MessageEvent| {
            // Get the data from our event
            let data = event.data();

            // Our data will be an array so cast it to an array
            let args = data.unchecked_ref::<js_sys::Array>();

            // The first argument will be the raw buffer of the UUID for a job that has completed
            // running
            let uuid_arg: Vec<u8> =
                Uint8Array::new(args.get(0).unchecked_ref::<ArrayBuffer>()).to_vec();
            // Read the raw UUID bytes into a UUID
            let uuid = unsafe { ptr::read_unaligned(uuid_arg.as_ptr() as *const Uuid) };

            // The next argument will be the raw buffer of the return value for complted job
            let data: Vec<u8> =
                Uint8Array::new(args.get(1).unchecked_ref::<ArrayBuffer>()).to_vec();

            // Using the job UUID obtain the sender that can be used to send the result
            let mut map = TASK_POOL.job_result_senders.lock().unwrap();

            let sender = map.remove(&uuid).expect("Unexpected job ID completed");

            // Kick of the send operation in an async task
            wasm_bindgen_futures::spawn_local(async move {
                sender
                    .send(data)
                    .await
                    .expect("Could not send worker response over channel");
            });
        }) as Box<dyn FnMut(MessageEvent)>);

        // Set the message listener to our callback
        worker.set_onmessage(Some(worker_callback.as_ref().unchecked_ref()));

        // Return the worker
        Self {
            _worker_callback: worker_callback,
            worker: Mutex::new(worker),
            job_result_senders: Default::default(),
        }
    }

    /// An internal helper method to do the actual task spawning
    ///
    /// This is just a shim so that users don't have to manually talk to the TASK_POOL static.
    async fn actually_spawn<D, R>(&self, function: fn(D) -> R, data: D) -> R
    where
        D: Send + Clone + 'static,
        R: Send + Clone + 'static,
    {
        // Create the array of Transferables to send to the worker
        let array = js_sys::Array::new();

        // The first arg is the job ID
        let job_id = Uuid::new_v4();
        // Get the raw byte buffer for that job ID
        let job_id_buffer = unsafe { job_id.copy_mem_bytes_to_new_arraybuffer() };
        // Push the buffer to our argument array
        array.push(&job_id_buffer.buffer());

        // The second arg is the pointer to the job wrapper function
        let wrapper_function_ptr: unsafe fn(fn(D) -> R, *mut u8, *mut u8) = worker_job_wrapper;
        let wrapper_function_ptr_usize = wrapper_function_ptr as usize;
        let wrapper_function_ptr_buffer =
            unsafe { wrapper_function_ptr_usize.copy_mem_bytes_to_new_arraybuffer() };
        array.push(&wrapper_function_ptr_buffer.buffer());

        // The third arg is the job function pointer
        let function_ptr: fn(D) -> R = function;
        let function_ptr_usize = function_ptr as usize;
        // Get the raw buffer of that functions pointer
        let function_ptr_buffer = unsafe { function_ptr_usize.copy_mem_bytes_to_new_arraybuffer() };
        // Add it to our arguments
        array.push(&function_ptr_buffer.buffer());

        // The fourth arg is the raw bytes of our job's data argument
        let data_buffer = unsafe { data.copy_mem_bytes_to_new_arraybuffer() };
        // And add it to our arguments
        array.push(&data_buffer.buffer());

        // Get the size of the return value of our job
        let ret_size = mem::size_of::<R>();
        // Get the raw bytes of that usize
        let ret_size_buffer = unsafe { ret_size.copy_mem_bytes_to_new_arraybuffer() };
        // And add it to our arguments
        array.push(&ret_size_buffer.buffer());

        // Create a channel that we will send the function result over
        let (sender, receiver) = async_channel::bounded(1);

        // Add the sender to our pending job senders
        self.job_result_senders
            .lock()
            .unwrap()
            .insert(job_id, sender);

        // Then we post our data to the worker
        self.worker
            .lock()
            .unwrap()
            .post_message_with_transfer(&array, &array)
            .expect("Could not send message to worker");

        // And we wait for a response from the worker with the raw bytes of our return type
        let ret = receiver
            .recv()
            .await
            .expect("Could not receive worker response over channel");

        // And copy it to our return type
        let ret = unsafe { ptr::read_unaligned(ret.as_ptr() as *const R) };

        // And return our return type
        ret
    }

    /// Spawn a blocking task on the worker pool and await the result
    pub async fn spawn<D, R>(function: fn(D) -> R, data: D) -> R
    where
        D: Send + Clone + 'static,
        R: Send + Clone + 'static,
    {
        TASK_POOL.actually_spawn(function, data).await
    }
}

unsafe fn worker_job_wrapper<D: Send + Clone, R: Send + Clone>(
    function: fn(D) -> R,
    data: *mut u8,
    ret_out: *mut u8,
) {
    let data = ptr::read_unaligned(data as *const D);
    let ret = function(data);
    ptr::write(ret_out as *mut R, ret);
}

/// Helper struct that allows us to return our worker callback to JavaScript so that it will handle
/// the garbage collection of the callback.
#[wasm_bindgen]
#[doc(hidden)]
pub struct WorkerCallback(Closure<dyn FnMut(MessageEvent)>);

#[wasm_bindgen]
#[doc(hidden)]
pub fn start_worker_pool_worker() -> WorkerCallback {
    // Get the global worker scope
    let worker = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>();

    // Create the callback that we will run when getting messages from parent
    let worker_callback = Closure::wrap(Box::new(|event: MessageEvent| {
        // Get the global worker scope
        let worker = js_sys::global().unchecked_into::<DedicatedWorkerGlobalScope>();

        // Get the data out of the message event
        let data = event.data();

        // We know that the data is an array of arguments so cast it to an array
        let args = data.unchecked_ref::<js_sys::Array>();

        // The first argument will be an arraybuffer that represents the ID of the job we need to
        // run. Get it's bytes into a Uint8Array
        let job_id_arg = args.get(0);

        // The second argument with be the bytes of a pointer to the wrapper job function. Get a
        // Vec<u8> from the buffer.
        let wrapper_func_pointer_arg: Vec<u8> =
            Uint8Array::new(args.get(1).unchecked_ref::<ArrayBuffer>()).to_vec();
        // Read it to a function pointer
        let wrapper_func_pointer_usize =
            unsafe { ptr::read_unaligned(wrapper_func_pointer_arg.as_ptr() as *const usize) };
        let wrapper_func_pointer: fn(usize, *mut u8, *mut u8) =
            unsafe { mem::transmute(wrapper_func_pointer_usize) };

        // The third argument will be the bytes of a pointer to job function we need to run. Get a
        // Vec<u8> from the buffer.
        let func_pointer_arg: Vec<u8> =
            Uint8Array::new(args.get(2).unchecked_ref::<ArrayBuffer>()).to_vec();

        // Read it to a usize ( we don't need to convert it to a pointer, because we will be passing
        // it to the wrapper function as an opaque usize that takes the place of the pointer. We're
        // kind of lying to rust, but it should be OK )
        let func_pointer_usize =
            unsafe { ptr::read_unaligned(func_pointer_arg.as_ptr() as *const usize) };

        // The fourth argument will be the raw bytes of the data argument to our job function. Get a
        // Vec<u8> from the buffer.
        let mut data_arg: Vec<u8> =
            Uint8Array::new(args.get(3).unchecked_ref::<ArrayBuffer>()).to_vec();

        // The fifth argument will be the raw bytes of the usize representing the size of the
        // return value of the job function.
        let ret_size_arg: Vec<u8> =
            Uint8Array::new(args.get(4).unchecked_ref::<ArrayBuffer>()).to_vec();
        // Read it to a usize
        let ret_size = unsafe { ptr::read_unaligned(ret_size_arg.as_ptr() as *const usize) };

        // Allocate a spot for the return value
        let mut ret = vec![0u8; ret_size];

        // Call our wrapper job function, passing it the job function pointer, the data pointer and
        // the return value pointer.
        wrapper_func_pointer(func_pointer_usize, data_arg.as_mut_ptr(), ret.as_mut_ptr());

        // Create a JavaScript buffer for the return value data
        let ret_buffer = Uint8Array::new_with_length(ret_size as u32);
        // Copy the data from the return value into the buffer
        ret_buffer.copy_from(ret.as_slice());

        // Create an array of arguments we will send back to the worker pool
        let array = Array::new();

        // The first argument is the job ID that we have completed
        // Push the buffer to our argument array
        array.push(&job_id_arg);

        // The second argument is the return value buffer
        array.push(&ret_buffer.buffer());

        // Send the message to the work pool
        worker
            .post_message_with_transfer(&array, &array)
            .expect("Could not send worker result to parent");
    }) as Box<dyn FnMut(MessageEvent)>);

    worker.set_onmessage(Some(worker_callback.as_ref().unchecked_ref()));

    WorkerCallback(worker_callback)
}
