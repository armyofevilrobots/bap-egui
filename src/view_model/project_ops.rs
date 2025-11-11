use super::BAPViewModel;

pub trait project_ops {
    fn save_project(&mut self); // Saves to known filename
    fn save_project_as_new(&mut self); // Save to a new file
    fn open_project(&mut self); // Open an existing project... Why the fuck am I commenting this obvious shit?!
}

impl project_ops for BAPViewModel {
    fn save_project(&mut self) {
        todo!()
    }

    fn save_project_as_new(&mut self) {
        todo!()
    }

    fn open_project(&mut self) {
        todo!()
    }
}
