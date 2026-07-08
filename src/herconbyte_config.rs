// herconbyte_config.rs
//
// White-label: TODOS los valores especificos de Herconbyte viven aca, en el crate
// principal (editable), NO en hbb_common (que queda pristino = byte-identico a upstream,
// asi el proximo bump de version es solo `git checkout <tag>` en el submodulo/vendored,
// sin re-aplicar parches).
//
// Cada valor se puede overridear por VARIABLE DE ENTORNO DE COMPILACION (option_env!):
// pasando HB_APP_NAME / HB_RENDEZVOUS / HB_KEY / HB_API en el build se despliega otro
// cliente sin tocar el codigo. Si no se pasan, el default es Herconbyte (asi el build
// actual funciona igual que siempre y NUNCA queda un cliente sin servidor).

/// Nombre visible de la app (barra de menu, About, carpeta de config/logs).
pub const APP_NAME: &str = match option_env!("HB_APP_NAME") {
    Some(v) => v,
    None => "Soporte Herconbyte",
};

/// Servidor rendezvous/relay (hbbs).
pub const RENDEZVOUS_SERVER: &str = match option_env!("HB_RENDEZVOUS") {
    Some(v) => v,
    None => "soporte.herconbyte.com.ar",
};

/// Clave publica del servidor (base64). Debe matchear la del hbbs.
pub const RS_PUB_KEY: &str = match option_env!("HB_KEY") {
    Some(v) => v,
    None => "hP6kEXdyMUxYSie5U3gYKbXqywJ2NQZ3RxNHkn2c6E4=",
};

/// API server (panel lejianwen / auto-update / address book).
pub const API_SERVER: &str = match option_env!("HB_API") {
    Some(v) => v,
    None => "https://api-soporte.herconbyte.com.ar",
};

/// Endpoint de chequeo de version (auto-update). Devuelve {"url":".../soporte/<version>"}.
pub const UPDATE_URL: &str = match option_env!("HB_UPDATE_URL") {
    Some(v) => v,
    None => "https://herconbyte.com.ar/soporte-version.php",
};

/// Aplica la config white-label en runtime, escribiendo los RwLock que hbb_common
/// ya expone como `pub` en upstream. NO toca hbb_common ni accede a Config (solo
/// escribe RwLocks), asi no dispara la carga lazy del storage.
///
/// IMPORTANTE: debe llamarse como PRIMERA cosa de cada entrypoint de proceso, ANTES
/// de cualquier acceso a Config, porque APP_NAME define la carpeta de config/logs y
/// se lee (lazy) en el primer acceso. Ver core_main() y el init del bridge Flutter.
pub fn apply() {
    // APP_NAME primero (define paths de config/logs).
    if let Ok(mut w) = hbb_common::config::APP_NAME.write() {
        *w = APP_NAME.to_owned();
    }
    // Servidor rendezvous con prioridad ABSOLUTA (EXE_RENDEZVOUS_SERVER gana sobre la
    // option del usuario y sobre PROD_/const). Para una herramienta de soporte cautivo
    // queremos que el server sea SIEMPRE el nuestro.
    if let Ok(mut w) = hbb_common::config::EXE_RENDEZVOUS_SERVER.write() {
        *w = RENDEZVOUS_SERVER.to_owned();
    }
    // La KEY y el API server no se setean aca: se resuelven directo desde las consts de
    // arriba en get_key() (src/common.rs) y get_api_server_() (src/common.rs), que son
    // codigo editable del crate principal. Ver esos edits.
}
