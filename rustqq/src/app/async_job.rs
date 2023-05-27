#![allow(dead_code, unused)]
use chrono::{DateTime, Local};
use cron::Schedule;
use std::{future::Future, pin::Pin, task::Poll};
use uuid::Uuid;

type JobFuture = Box<dyn Future<Output = ()> + Send + 'static>;
pub struct AsyncJob {
    schedule: Schedule,
    run: Option<Box<dyn PinedFuture + Send>>,
    job_id: Uuid,
    last_tick: Option<DateTime<Local>>,
}
trait PinedFuture {
    fn get_pined(&mut self) -> Pin<JobFuture>;
}
struct JobWrapper<F, T>
where
    F: FnMut() -> T,
    T: Future,
{
    f: F,
}
impl<F, T> JobWrapper<F, T>
where
    F: FnMut() -> T,
    T: Future,
{
    fn new(f: F) -> Self {
        Self { f }
    }
}
impl<F, T> PinedFuture for JobWrapper<F, T>
where
    F: FnMut() -> T,
    T: Future<Output = ()> + Send + 'static,
{
    fn get_pined(&mut self) -> Pin<JobFuture> {
        Box::pin((self.f)())
    }
}

impl AsyncJob {
    pub fn new<F, T>(schedule: Schedule, run: F) -> Self
    where
        F: 'static + FnMut() -> T + Send,
        T: 'static + Future<Output = ()> + Send,
    {
        Self {
            schedule,
            run: Some(Box::new(JobWrapper::new(run))),
            job_id: Uuid::new_v4(),
            last_tick: None,
        }
    }
    pub fn excute(&mut self, now: &DateTime<Local>) -> Option<Pin<JobFuture>> {
        if self.last_tick.is_none() {
            self.last_tick = Some(*now);
            return None;
        }
        #[allow(clippy::never_loop)]
        for event in self.schedule.after(&self.last_tick.unwrap()).take(1) {
            if event > *now {
                return None;
            }

            self.last_tick = Some(event);
            let rv = self.run.as_mut().map(|f| f.get_pined());
            return rv;
        }
        None
    }
}
#[derive(Default)]
pub struct AsyncJobScheduler {
    jobs: Vec<AsyncJob>,
}
impl AsyncJobScheduler {
    pub fn new() -> Self {
        self::AsyncJobScheduler::default()
    }
    pub fn add_job(mut self, job: AsyncJob) -> Self {
        self.jobs.push(job);
        self
    }
    pub fn add_jobs(mut self, jobs: Vec<AsyncJob>) -> Self {
        self.jobs.extend(jobs);
        self
    }
    pub fn run_pending(&mut self) -> AsyncSchedulerFuture {
        let now = Local::now();
        let mut futures = Vec::new();
        for job in &mut self.jobs {
            if let Some(future) = job.excute(&now) {
                futures.push(Some(future));
            }
        }
        AsyncSchedulerFuture { futures }
    }
    pub async fn run(mut self) {
        actix_web::rt::spawn(async move {
            loop {
                self.run_pending().await;
            }
        });
    }
}

pub struct AsyncSchedulerFuture {
    futures: Vec<Option<Pin<JobFuture>>>,
}

impl Future for AsyncSchedulerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut all_done = true;

        for future in &mut self.get_mut().futures {
            if let Some(this_future) = future {
                if this_future.as_mut().poll(cx) == Poll::Ready(()) {
                    future.take();
                } else {
                    all_done = false;
                }
            }
        }
        if all_done {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
