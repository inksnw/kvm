#[macro_use]
extern crate rocket;

use std::error::Error;

use rocket::http::Method;
use rocket::response::content;
use rocket_cors::{AllowedOrigins, CorsOptions};
use virt::connect;
use virt::connect::Connect;

use models::InsModel;

mod models;


fn disconnect(mut conn: Connect) {
    if let Err(e) = conn.close() {
        panic!("Failed to disconnect from hypervisor: code {}, message: {}", e.code, e.message);
    }
    println!("Disconnected from hypervisor");
}

#[get("/")]
fn list() -> content::Json<String> {
    let flags = connect::VIR_CONNECT_LIST_DOMAINS_ACTIVE | connect::VIR_CONNECT_LIST_DOMAINS_INACTIVE;

    let conn = get_conn();

    let mut v: Vec<InsModel> = Vec::new();

    if let Ok(num_active_domains) = conn.num_of_domains() {
        if let Ok(num_inactive_domains) = conn.num_of_defined_domains() {
            println!("There are {} active and {} inactive domains", num_active_domains, num_inactive_domains);
        }
    }
    if let Ok(doms) = conn.list_all_domains(flags) {
        for dom in doms {
            let id = dom.get_id().unwrap_or(0);
            let name = dom.get_name().unwrap_or_else(|_| String::from("no-name"));
            let active = dom.is_active().unwrap_or(false);
            if let Ok(dinfo) = dom.get_info() {
                let tmp = InsModel {
                    ins_id: id,
                    state: dinfo.state,
                    ins_name: name,
                    cpu: dinfo.nr_virt_cpu,
                    mem: dinfo.memory / 1024 / 1024,
                    active,
                };
                v.push(tmp)
            }
        }
    }
    content::Json(serde_json::to_string(&v).unwrap())
}

fn get_conn() -> Connect {
    let uri = String::from("qemu+ssh://root@192.168.50.20/system?socket=/var/run/libvirt/libvirt-sock");
    println!("Attempting to connect to hypervisor: '{}'", uri);

    let conn = match Connect::open(&uri) {
        Ok(c) => c,
        Err(e) => panic!(
            "No connection to hypervisor: code {}, message: {}",
            e.code, e.message
        ),
    };

    match conn.get_uri() {
        Ok(u) => println!("Connected to hypervisor at '{}'", u),
        Err(e) => {
            disconnect(conn);
            panic!("Failed to get URI for hypervisor connection: code {}, message: {}", e.code, e.message);
        }
    };
    return conn;
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true)
        .to_cors()?;
    rocket::build()
        .mount("/", routes![list])
        .attach(cors)
        .launch()
        .await?;
    Ok(())
}

