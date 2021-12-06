use std::process::Termination;

pub(crate) enum Exit {
    Me(String),
    Program(i32),
}

impl Termination for Exit {
    fn report(self) -> i32 {
        match self {
            Exit::Me(e) => {
                eprintln!("{}", e);
                1
            }
            Exit::Program(code) => code,
        }
    }
}
