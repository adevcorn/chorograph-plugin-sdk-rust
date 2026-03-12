pub trait AIProvider {
    fn id(&self) -> String;
    fn display_name(&self) -> String;
    fn get_models(&self) -> Vec<String>;
    fn send_message(&self, session_id: &str, text: &str) -> Result<(), String>;
}
