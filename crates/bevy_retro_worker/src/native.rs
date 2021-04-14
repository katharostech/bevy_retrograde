use bevy_tasks::TaskPool;
use lazy_static::lazy_static;

lazy_static! {
    static ref TASK_POOL: BlockingTaskPool = BlockingTaskPool::create();
}

pub struct BlockingTaskPool {
    task_pool: TaskPool,
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
        Self {
            task_pool: TaskPool::default(),
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
        self.task_pool
            .spawn(async move {
                function(data)
            })
            .await
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
