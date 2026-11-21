pub mod view {
    pub mod login;
}

#[derive(Debug, Default)]
pub enum Views {
    #[default]
    Login,
    MainDashboard,
    Stats,
    Details,
}
