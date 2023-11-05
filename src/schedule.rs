use alloc::boxed::Box;
use alloc::collections::VecDeque;
use sam3x8e::RTT;

use crate::config::RTT_PRESCALER;

type TaskClosure = dyn Fn(&mut Scheduler);

pub struct Task {
    pub task: Box<TaskClosure>,
    pub time: u32,
}

pub struct Scheduler {
    tasks: VecDeque<Task>,
    last_task: Option<Box<TaskClosure>>,
    rtt: RTT,
}

impl Scheduler {
    pub fn new(rtt: RTT) -> Self {
        rtt.mr
            .write_with_zero(|w| unsafe { w.rtpres().bits(RTT_PRESCALER) });
        Scheduler {
            tasks: VecDeque::new(),
            last_task: None,
            rtt,
        }
    }

    pub fn push_box(&mut self, task: Box<TaskClosure>, ms: u32) {
        self.push_task(Task {
            task,
            time: self.rtt.vr.read().bits() + ms,
        });
    }

    pub fn push<F>(&mut self, task: F, ms: u32)
    where
        F: Fn(&mut Scheduler) + 'static,
    {
        self.push_task(Task {
            task: Box::new(task),
            time: self.rtt.vr.read().bits() + ms,
        });
    }

    pub fn push_task(&mut self, task: Task) {
        for (index, current_task) in self.tasks.iter().enumerate() {
            if task.time < current_task.time {
                self.tasks.insert(index, task);
                break;
            }
        }
    }

    #[inline(always)]
    fn pop_next_task(&mut self) -> Option<Task> {
        self.tasks.pop_front()
    }

    pub fn yield_for(&mut self, ms: u32) {
        let until = self.rtt.vr.read().bits() + ms;
        while self.rtt.vr.read().bits() < until {
            if let Some(task) = self.pop_next_task() {
                (task.task)(self);
                self.last_task = Some(task.task);
            }
        }
    }

    #[inline(always)]
    pub fn main_loop(mut self) -> ! {
        loop {
            if let Some(task) = self.pop_next_task() {
                (task.task)(&mut self);
                self.last_task = Some(task.task);
            }
        }
    }

    pub fn repeat_in(&mut self, ms: u32) {
        let task = self.last_task.take();
        self.push_box(task.unwrap(), ms);
    }
}
