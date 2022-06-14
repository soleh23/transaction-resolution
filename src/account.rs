#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Account {
    pub client_id: u16,
    pub available: f32,
    pub held: f32,
    pub locked: bool,
}

impl Account {
    pub fn new_with_client(client_id: u16) -> Account {
        Account {
            client_id,
            available: 0.0,
            held: 0.0,
            locked: false,
        }
    }
    pub fn new(client_id: u16, available: f32, held: f32, locked: bool) -> Account {
        Account {
            client_id,
            available,
            held,
            locked,
        }
    }
}
