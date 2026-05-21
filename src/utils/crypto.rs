use aes::cipher::block_padding::Pkcs7;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as base64;
use rand_core::{OsRng, RngCore};
use rsa::pkcs8::DecodePrivateKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

use crate::config::FRONTEND_RSA_PRIVATE_KEY;
use crate::result::{AppError, ThrowError};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const PASS_PHRASE: &str = "qnxg-crypto-2023";

/// 生成一个长度为8的随机salt
#[inline]
fn gen_salt() -> [u8; 8] {
    let mut rng = OsRng;
    let mut bytes = [0u8; 8];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// openssl的加密方式，使用md5生成Aes Cbc 256的key和iv，以下是伪代码
/// ```text
/// hash1_128 = MD5(Passphrase + Salt)
/// hash2_128 = MD5(hash1_128 + Passphrase + Salt)
/// hash3_128 = MD5(hash2_128 + Passphrase + Salt)
/// Key = hash1_128 + hash2_128
/// IV  = hash3_128;
/// ```
#[inline]
fn passphrase_to_key_and_iv(
    salt: &[u8],
    pass_phrase: &str,
) -> ([u8; 32], [u8; 16]) {
    assert_eq!(salt.len(), 8);
    let hash1 = md5::compute([pass_phrase.as_bytes(), salt].concat());
    let hash2 = md5::compute(
        [hash1.as_slice(), pass_phrase.as_bytes(), salt].concat(),
    );
    let hash3 = md5::compute(
        [hash2.as_slice(), pass_phrase.as_bytes(), salt].concat(),
    );
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];
    let temp = [hash1.as_slice(), hash2.as_slice()].concat();
    key.copy_from_slice(&temp);
    iv.copy_from_slice(hash3.as_slice());
    (key, iv)
}

/// 加密函数，采用Aes256Cbc加密和Pkcs7填充
pub fn encrypt(data: &str) -> String {
    // 生成一个长度为8的随机字符串
    let salt = gen_salt();

    let (key, iv) = passphrase_to_key_and_iv(&salt, PASS_PHRASE);
    let key = GenericArray::from_slice(key.as_slice());
    let iv = GenericArray::from_slice(iv.as_slice());
    let res = Aes256CbcEnc::new(key, iv)
        .encrypt_padded_vec_mut::<Pkcs7>(data.as_bytes());
    // 添加 Salted__ 和 salt 前缀
    let prefix = b"Salted__";
    let res = [prefix, &salt, res.as_slice()].concat();
    base64.encode(&res)
}

/// 解密函数，可能会返回错误
pub fn decrypt(
    data: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let decode = base64.decode(data)?;
    if decode.len() < 16 {
        return Err("Invalid data length".into());
    }
    let salt = &decode[8..16];
    let (key, iv) = passphrase_to_key_and_iv(salt, PASS_PHRASE);
    let key = GenericArray::from_slice(&key);
    let iv = GenericArray::from_slice(&iv);
    let res = Aes256CbcDec::new(key, iv)
        .decrypt_padded_vec_mut::<Pkcs7>(&decode[16..])?;
    Ok(String::from_utf8(res)?)
}

/// 解密前端 node-forge RSAES-PKCS1-V1_5 加密的 Base64 密文
pub fn decrypt_frontend(data: &str) -> Result<String, AppError> {
    let private_key =
        RsaPrivateKey::from_pkcs8_pem(&FRONTEND_RSA_PRIVATE_KEY)
            .throw_error("解析私钥失败")?;
    let cipher = base64.decode(data).throw_error("解码密文失败")?;
    let plain = private_key
        .decrypt(Pkcs1v15Encrypt, &cipher)
        .throw_error("解密失败")?;
    String::from_utf8(plain).throw_error("解密失败")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = "";
        let encrypted = encrypt(data);
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_decrypt_frontend() {
        let data = "YN7iKaCd2WrxL3yZla9EqjzqQaHV17SYuU58NwMFwpCdIs3yRiIpTeMZmSVbP6quB0myLAem/5lQ+YJxdlkwgBUHYuX8Jx0M12Ef1nELg+dXyDyXzgthETE7PO+Z5ZFOtPbyyyl1/FcFNc4ItED69FGyarzzSCFnei/2yXN8uzwmTPWp6bJ72T9cwr78zw49CkYDlAQwnkv9BU/EEeSURZr3OxAboz45F8Pio2UhuFdnZ3q5CNbp2+qxxlGZ+RyABK+dV0qivTg+f5rib2sIvQ2Rxave5KILizP4cfjfizTjsrXatxXrcu6Hxf2S8pG4CGS5D/11/YBxn4Md3V2brQ==";
        let decrypted = decrypt_frontend(data).unwrap();
        assert_eq!(decrypted, "11111111111111111111");
    }
}
