
mod google;
mod doctor;
mod surgeon;
mod hospital;

pub use google::generate_request_jwt;
pub use doctor::check_health;
pub use surgeon::ressurect;
pub use hospital::observe;

//health - check -> Verifica se esta on -> First Fase Completed
//ressurect -> Reinicia Processo -> Reinicia o processo Completed
//obeserve -> Monitora Ngrok
//propagate -> Atualiza remote config