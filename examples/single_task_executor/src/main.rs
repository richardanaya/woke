use once_cell::sync::OnceCell;
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
    shared_state: Arc<Mutex<SharedState>>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                std::mem::drop(shared_state);
                waker.wake()
            }
        });

        TimerFuture { shared_state }
    }
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

struct Executor {
    task: Option<Arc<Task>>,
}

struct Task {
    pub future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
}

impl Woke for Task {
    fn wake_by_ref(_: &Arc<Self>) {
        Executor::run()
    }
}

impl Executor {
    fn spawn(future: impl Future<Output = ()> + 'static + Send) {
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(future))),
        });
        let mut e = get_executor().lock().unwrap();
        e.task = Some(task);
    }
    fn run() {
        let e = get_executor().lock().unwrap();
        if let Some(task) = &e.task {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                if let Poll::Pending = future.as_mut().poll(context) {
                    *future_slot = Some(future);
                }
            }
        }
    }
}

fn get_executor() -> &'static Mutex<Executor> {
    static INSTANCE: OnceCell<Mutex<Executor>> = OnceCell::new();
    INSTANCE.get_or_init(|| Mutex::new(Executor { task: None }))
}

fn main() {
    Executor::spawn(async {
        println!("howdy!");
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
        std::process::exit(0);
    });
    loop {
        Executor::run();
    }
}
