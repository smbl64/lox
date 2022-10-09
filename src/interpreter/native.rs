use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpret: &mut Interpreter,
        _arguments: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        let start = SystemTime::now();
        let since_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backward");

        Ok(Object::Number(since_epoch.as_millis() as f64 / 1000.0))
    }
}

pub fn clock() -> Rc<dyn Callable> {
    Rc::new(Clock)
}