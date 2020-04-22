use crate::models::Phase;

#[derive(Debug, Fail)]
pub enum PhaseError {
    #[fail(display = "fail to get from DB")]
    DbError,
    #[fail(display = "wrong phases sequence")]
    SequenceError,
    #[fail(display = "empty sequence")]
    Empty,
}

pub struct Phases {
    phases: Vec<Phase>,
    pub count: usize,
}

impl Phases {
    pub fn new(mut phases: Vec<Phase>) -> Result<Phases, PhaseError> {
        let count = phases.len();
        phases.sort_by(|a, b| a.number.cmp(&b.number));
        let mut i: i32 = phases.first().ok_or(PhaseError::Empty)?.number;
        for ph in phases[1..].iter() {
            if i + 1 != ph.number {
                return Err(PhaseError::SequenceError);
            }
            i = ph.number;
        }

        Ok(Phases { phases, count })
    }

    pub fn get(&self, phase_num: i32) -> Option<&Phase> {
        self.phases.iter().find(|&ph| ph.number == phase_num)
    }
}
