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

/// Windows: sacarle el freno `stop-service` a la config DEL SERVICIO.
///
/// Sintoma que arregla: la app dice "El servicio no se esta ejecutando" con el
/// servicio de Windows En ejecucion, la contrasena de un solo uso vacia y el equipo
/// sin aparecer en la libreta. Caso real del 21/09/26 en la maquina de un cliente:
/// dos ciclos de desinstalar e instalar no lo arreglaron.
///
/// Por que pasa: la config de opciones (`<APP_NAME>2.toml`) tiene DOS copias por
/// maquina, la del usuario que instala y la del servicio -- que corre como
/// LocalSystem, pero hbb_common redirige ese perfil a ServiceProfiles\LocalService
/// (ver `patch()` en hbb_common/src/config.rs). Desinstalar escribe
/// `stop-service = 'Y'` y NO borra la carpeta del servicio; al reinstalar,
/// `install_service()` limpia la marca con `Config::set_option`, que escribe la
/// copia del USUARIO. El servicio nuevo arranca, lee la SUYA, ve el freno y no se
/// registra contra el servidor (rendezvous_mediator.rs), mientras el SCM lo muestra
/// corriendo y la app lee esa misma opcion por IPC y pinta el cartel.
///
/// El `--import-config` de upstream no lo tapa: solo copia si el archivo del usuario
/// es MAS NUEVO, y un `set_option` a vacio no reescribe nada cuando la clave ya no
/// estaba de ese lado.
///
/// Se saca SOLO esa clave: el resto de las opciones del servicio se conservan.
/// Borrar el archivo dejaria al equipo sin configuracion.
#[cfg(windows)]
pub fn limpiar_freno_del_servicio() {
    let raiz = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_owned());
    let archivo = std::path::Path::new(&raiz)
        .join("ServiceProfiles")
        .join("LocalService")
        .join("AppData")
        .join("Roaming")
        .join(APP_NAME)
        .join("config")
        .join(format!("{}2.toml", APP_NAME));
    // Instalacion limpia: el archivo todavia no existe y no hay nada que sacar.
    let Ok(texto) = std::fs::read_to_string(&archivo) else {
        return;
    };
    if !texto.contains("stop-service") {
        return;
    }
    let mut salida = String::with_capacity(texto.len());
    for linea in texto.lines() {
        // La clave puede estar pelada (`stop-service = 'Y'`) o entrecomillada.
        if linea
            .trim_start()
            .trim_start_matches('"')
            .starts_with("stop-service")
        {
            continue;
        }
        salida.push_str(linea);
        salida.push('\n');
    }
    match std::fs::write(&archivo, salida) {
        Ok(()) => hbb_common::log::info!("saque stop-service de {:?}", archivo),
        Err(e) => hbb_common::log::warn!("no pude limpiar stop-service en {:?}: {}", archivo, e),
    }
}
