use {
    std::{
        future::Future,
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
        thread,
        time::Duration,
    },
    woke::{waker_ref, Woke},
};

pub struct TimerFuture {
    state: Arc<Mutex<TimerFutureState>>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();

        // if we are done, return Ready!
        if state.completed {
            Poll::Ready(())
        } else {
            // if not, store that waker for later when we need it!
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        // create some state for us to pass around safely!
        let state = Arc::new(Mutex::new(TimerFutureState {
            completed: false,
            waker: None,
        }));

        let thread_state = state.clone();
        thread::spawn(move || {
            // wait a bit
            thread::sleep(duration);
            let mut state = thread_state.lock().unwrap();

            //mark as complete
            state.completed = true;

            // get that waker and wake our task up!
            if let Some(waker) = state.waker.take() {
                std::mem::drop(state);
                waker.wake()
            }
        });

        TimerFuture { state }
    }
}

struct TimerFutureState {
    completed: bool,
    waker: Option<Waker>,
}

// our executor just holds one task
struct Executor {
    task: Option<Arc<Task>>,
}

// Our task holds onto a future the executor can poll
struct Task {
    pub future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
}

// specify how we want our tasks to wake up
impl Woke for Task {
    fn wake_by_ref(_: &Arc<Self>) {
        // run the executor again because something finished!
        Executor::run()
    }
}

impl Executor {
    fn spawn(future: impl Future<Output = ()> + 'static + Send) {
        // store our task in global state
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
        });
        let mut e = EXECUTOR.lock().unwrap();
        e.task = Some(task);

        // we drop this early because otherwise run() will cause a mutex lock
        std::mem::drop(e);

        // get things going!
        Executor::run();
    }
    fn run() {
        // get our task from global state
        let e = EXECUTOR.lock().unwrap();
        if let Some(task) = &e.task {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // make a waker for our task
                let waker = waker_ref(&task);
                // poll our future and give it a waker
                let context = &mut Context::from_waker(&*waker);
                if let Poll::Pending = future.as_mut().poll(context) {
                    *future_slot = Some(future);
                }
            }
        }
    }
}

static EXECUTOR: Mutex<Executor> = Mutex::new(Executor { task: None });

fn main() {
    Executor::spawn(async {
        println!("howdy!");
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");

        // now we can exit
        std::process::exit(0);
    });

    // prevent exiting immediately
    loop {}
}
