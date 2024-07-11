use super::{State, Store};

pub fn create_app_state() -> Store<'static> {
  let fields: Vec<(String, State)> = Vec::from([
    ("test_field".into(), State::Integer(0))
  ]);
  Store::create(fields)
}