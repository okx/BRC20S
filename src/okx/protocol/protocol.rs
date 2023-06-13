pub trait Protocol {
    fn execute_message(&mut self) -> ();
}

