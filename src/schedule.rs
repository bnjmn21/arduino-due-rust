use crate::sam3x8e::rtt::Rtt;

pub struct Scheduler<'a> {
    rtt: &'a Rtt,
}

impl Scheduler<'_> {
    pub fn new(rtt: &Rtt) -> Scheduler {
        Scheduler { rtt }
    }
}
