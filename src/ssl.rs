use std::{
    error::Error,
    fs::{self, File},
    io::Write,
};

use openssl::{
    base64,
    ec::{EcGroup, EcKey},
    nid::Nid,
};

const PRIV_FILE: &str = "data/private.pem";
const PUB_FILE: &str = "data/public.b64";

pub(crate) fn ensure_keys() -> Result<(), Box<dyn Error>> {
    if fs::exists(PRIV_FILE)? && fs::exists(PUB_FILE)? {
    } else {
        generate_keys()?;
    }

    Ok(())
}

fn generate_keys() -> Result<(), Box<dyn Error>> {
    let nid = Nid::X9_62_PRIME256V1;
    let group = EcGroup::from_curve_name(nid)?;
    let key = EcKey::generate(&group)?;
    let private = String::from_utf8(key.private_key_to_pem()?)?;

    let mut f = File::create(PRIV_FILE)?;
    write!(f, "{}", private)?;

    let public = key.public_key_to_der().map(|x| {
        let n = x.len() - 65;
        x.into_iter().skip(n).collect::<Vec<_>>()
    })?;
    let out = base64::encode_block(&public);

    let mut f = File::create(PUB_FILE)?;
    write!(f, "{}", out)?;

    Ok(())
}
