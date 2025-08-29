#![allow(unused)]

///
/// - Argon2d：针对GPU破解攻击提供最强防护
/// - Argon2i：专为抵御侧信道攻击优化
/// - Argon2id：（默认方案）融合Argon2i与Argon2d优势的混合版本
///
#[cfg(test)]
mod argon2_test {
    use argon2::{
        password_hash::{
            rand_core::OsRng,
            PasswordHash, PasswordHasher, PasswordVerifier, SaltString
        },
        Argon2
    };
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;

    /// Example1: 使用默认设置创建密码哈希并验证它
    fn default_argon2(password: &str) ->  anyhow::Result<()> {

        let salt = SaltString::generate(&mut OsRng);

        // 具有默认参数的 Argon2 （Argon2id v19）
        let argon2 = Argon2::default();

        // Hash加密 ($argon2id$v=19$...)
        let password_hash = argon2.hash_password(password.as_ref(), &salt)?.to_string();

        println!("{}", password_hash);
        // 验证
        //
        // 注意：使用来自 'parsed_hash' 的哈希参数，而不是 'Argon2' 实例中配置的内容。
        let parsed_hash = PasswordHash::new(&password_hash)?;
        assert!(Argon2::default().verify_password(password.as_ref(), &parsed_hash).is_ok());
        Ok(())


    }

    /// Example1: 使用自定义设置创建密码哈希并进行验证：
    fn custom_argon2() ->  anyhow::Result<()> {
        let password = b"hunter42";
        let salt = b"example salt"; // 每个密码的盐应该不同

        let mut output_key_material = [0u8; 32]; // 可以是任何所需的尺寸

        println!("{:?}", output_key_material);
        println!("{:?}", output_key_material);
        println!("Hex: {}",hex::encode(output_key_material));
        println!("Base64: {}", BASE64_STANDARD.encode(&output_key_material));
        Argon2::default().hash_password_into(password, salt, &mut output_key_material)?;
        println!("{:?}", output_key_material);
        println!("Hex: {}",hex::encode(output_key_material));
        println!("Base64: {}", BASE64_STANDARD.encode(&output_key_material));
        Ok(())
    }


    #[test]
    fn async_base() {
        default_argon2("abcdefg");
        custom_argon2();
    }

    #[test]
    fn test_linked_list() {
    }
}

fn main() {}
