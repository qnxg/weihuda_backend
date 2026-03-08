use aes::cipher::block_padding::Pkcs7;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncryptMut, KeyIvInit};
use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as base64;
use rand_core::{OsRng, RngCore};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

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

#[cfg(test)]
mod tests {
    use super::*;
    use aes::cipher::BlockDecryptMut;
    use std::error::Error;

    type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

    fn decrypt(data: &str) -> Result<String, Box<dyn Error>> {
        let decode = base64.decode(data)?;
        let salt = &decode[8..16];
        let (key, iv) = passphrase_to_key_and_iv(salt, PASS_PHRASE);
        let key = GenericArray::from_slice(&key);
        let iv = GenericArray::from_slice(&iv);
        let res = Aes256CbcDec::new(key, iv)
            .decrypt_padded_vec_mut::<Pkcs7>(&decode[16..])?;
        Ok(String::from_utf8(res)?)
    }

    #[test]
    fn test_encrypt_decrypt() {
        let data = "";
        let encrypted = encrypt(data);
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_data_in_database() {
        let data = "";
        let decrypted = decrypt(data).unwrap();
        println!("{}", decrypted);
    }
}
