#![allow(unused)]
use crate::error::{Error, Rresult};

#[cfg(test)]
mod tests {
    use hmac::{digest::InvalidLength, Hmac, Mac};
    use serial_test::serial;
    use hex_literal::hex;
    use sha2::Sha512;
    use crate::Error;

    #[serial]
    #[tokio::test]
    async fn hex() {
        let expected = hex!("97d2a569059bbcd8ead4444ff99071f4c01d005bcefe0d3567e1be628e5fdcd9");
        println!("{:?}", expected);
        println!("{:?}", String::from_utf8(expected.to_vec()));
        
    }

    #[serial]
    #[tokio::test]
    async fn hmac_sha512_encrypt() -> Result<(), Error>{
        let key = b"1111111111111111";
        let content = b"111111111";
        let salt = b"11111111";

        // -- Create a HMAC-SHA-512 from key.
        let mut hmac_sha512 =
            Hmac::<Sha512>::new_from_slice(key).map_err(|ex: InvalidLength| Error::HmacInvalidLength(ex))?;

        // -- append content.
        // hmac_sha512.update();
        // hmac_sha512.update();

        // -- Finalize and b64u encode.
        // let hmac_result = hmac_sha512.finalize();
        // -- convert result to bytes
        // let result_bytes = hmac_result.into_bytes();

        // println!("--> {:?}", result_bytes);
        // let result = base64_url::encode(&result_bytes);

        // Ok(result)
        Ok(())
    }
}
