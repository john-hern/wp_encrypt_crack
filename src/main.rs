extern crate clap;
use clap::{Arg, App};
use std::u16;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;
use std::convert::From;

fn main(){
    let matches = App::new("Word Perfect Encryption Cracker")
                          .version("1.0")
                          .author("John hernandez <johnhe@indeed.com>")
                          .about("Cracks things")
                          .arg(Arg::with_name("crack_file")
                               .short("c")
                               .long("crack-file")
                               .value_name("FILE")
                               .help("File to crack")
                               .takes_value(true))
                          .arg(Arg::with_name("encrypt_file")
                               .short("e")
                               .long("encrypt-file")
                               .value_name("FILE")
                               .help("File to encrypt")
                               .takes_value(true))
                          .arg(Arg::with_name("decrypt_file")
                               .short("d")
                               .long("decrypt-file")
                               .value_name("FILE")
                               .help("File to decrypt")
                               .takes_value(true))
                          .arg(Arg::with_name("key")
                               .short("k")
                               .long("key")
                               .value_name("string")
                               .help("Key String")
                               .required(true)
                               .takes_value(true))
                          .arg(Arg::with_name("depth")
                               .long("depth")
                               .value_name("int")
                               .help("Depth of frequent chars to use")
                               .default_value("5")
                               .takes_value(true))
                          .arg(Arg::with_name("min_length")
                               .long("min")
                               .value_name("int")
                               .help("Min key length")
                               .default_value("3")
                               .takes_value(true))
                          .arg(Arg::with_name("max_length")
                               .long("max")
                               .value_name("int")
                               .help("Max key length")
                               .default_value("12")
                               .takes_value(true))    
                          .get_matches();
    
    if let Some(file) = matches.value_of("encrypt_file"){
        let key = matches.value_of("key").unwrap();
        encrypt_file(file, key.as_bytes());
    }
    if let Some(file) = matches.value_of("decrypt_file"){
        let key = matches.value_of("key").unwrap();
        if let Ok(decrypted) = decrypt_file(file, key.as_bytes()){
            decrypted.write_file(format!("{}.dec", file).as_ref())
        }
    }
    if let Some(file) = matches.value_of("crack_file"){
        let key = matches.value_of("key").unwrap();
        let depth = matches.value_of("depth").unwrap().parse::<usize>().unwrap();
        let min = matches.value_of("min_length").unwrap().parse::<usize>().unwrap();
        let max = matches.value_of("max_length").unwrap().parse::<usize>().unwrap();
        
        for x in min..max
        {
            let ret = crack(file ,x, depth, key.as_bytes()[0]).unwrap();
            for value in ret.iter(){
                println!("{}", value);
            }
        }   
    }
}
enum WPFile{
    Encrypted(WPEncryptedFile),
    Unencrypted(WPUnencryptedFile)
}

impl WPFile{
    fn from_file(file_path: &str) -> Result<WPFile, io::Error>{
        let bytes = get_file_bytes(file_path).unwrap();
        WPFile::from_raw(&bytes)
    }
    fn from_raw(bytes: &[u8]) -> Result<WPFile, io::Error>{
        if bytes.len() < 7 {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        //Let's pop off the header.. I dont feel like checking it right now.
        {
            if bytes[0] != 0xFF &&
                bytes[1] != 0xFF &&
                bytes[2] != 0x61 &&
                bytes[3] != 0x61{
                    return Err(io::Error::from(io::ErrorKind::InvalidData));
            }
        }
        let file_checksum = ((bytes[4] as u16) << 8) | bytes[5] as u16;

        if file_checksum == 0x0 {
            Ok(WPFile::Unencrypted(WPUnencryptedFile::new(&bytes)))
        }else{
            let content = bytes[6..].iter().copied().collect();
            Ok(WPFile::Encrypted(
                WPEncryptedFile{
                            magic_bytes: [0xFF, 0xFF, 0x61, 0x61],
                            checksum: file_checksum,
                            contents: content
                        }
            ))
        }
    }
}
struct WPUnencryptedFile {
    magic_bytes: [u8; 4],
    checksum: u16,
    contents: Vec<u8>
}
impl WPUnencryptedFile{
    fn new(contents: &[u8]) -> WPUnencryptedFile
    {
        WPUnencryptedFile{
            magic_bytes: [0xFF, 0xFF, 0x61, 0x61],
            checksum: 0,
            contents: contents.iter().copied().collect::<Vec<u8>>()
        }
    }
    fn encrypt(self, key: &[u8]) -> WPEncryptedFile
    {
        WPEncryptedFile::encrypt(&self.contents, key)
    }
    fn write_file(&self, file_name: &str){
        let mut contents: Vec<u8> = Vec::new(); 
        contents.extend_from_slice(&self.magic_bytes);
        contents.extend(self.checksum.to_be_bytes().iter());
        contents.extend(self.contents.iter());
        write_file_bytes(file_name, &contents).unwrap();
    }
    
}
struct WPEncryptedFile {
    magic_bytes: [u8; 4],
    checksum: u16,
    contents: Vec<u8>
}

impl WPEncryptedFile{
    fn encrypt(contents: &[u8], key: &[u8]) -> WPEncryptedFile
    {
        let encrypted = crypt(contents, key);
        let checksum = checksum(key);
        WPEncryptedFile{
            magic_bytes: [0xFF, 0xFF, 0x61, 0x61],
            checksum,
            contents: encrypted
        }
    }
    fn decrypt(self, key: &[u8]) -> WPUnencryptedFile
    {
        let decrypted = crypt(&self.contents, key);
        WPUnencryptedFile::new(&decrypted)
    }
    fn write_file(&self, file_name: &str){
        let mut contents: Vec<u8> = Vec::new(); 
        contents.extend_from_slice(&self.magic_bytes);
        contents.extend(self.checksum.to_be_bytes().iter());
        contents.extend(self.contents.iter());
        write_file_bytes(file_name, &contents).unwrap();
    }
}

fn encrypt_file(file_name: &str, key_bytes: &[u8]){
    println!("Encrypting file: {:?}", file_name);
    let bytes = get_file_bytes(file_name).unwrap();
    let encrypted =  WPUnencryptedFile::new(&bytes).encrypt(key_bytes);
    let name = file_name.to_owned() + ".enc";
    encrypted.write_file(&name);
}

fn decrypt_file(file_name: &str, key_bytes: &[u8]) -> Result<WPUnencryptedFile, io::Error>
{
    if let Ok(WPFile::Encrypted(file)) = WPFile::from_file(file_name){
        Ok(file.decrypt(key_bytes))
    }else{
        //Invalid file. 
        Err(io::Error::from(io::ErrorKind::InvalidInput))
    }
}

//This is the actual cracking function. 
fn crack(file_name: &str, key_size: usize, depth: usize, frequent_char: u8) -> Result<Vec<String>, io::Error>
{
    let mut file_checksum = 0;
    let mut bytes: Vec<u8> = Vec::new(); 
    if let Ok(WPFile::Encrypted(file)) = WPFile::from_file(file_name){
        bytes.extend(file.contents);
        file_checksum = file.checksum;
    }else{
        //Invalid file. 
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    
    let mut blocks: Vec<Vec<u8>> = Vec::new(); 
    
    let y = bytes.len()/key_size as usize;
    let x = key_size as usize;
    
    let mut sequence: u8 = key_size as u8 + 1; 
    //We are assuming the key size in this case. We need to "remove" the sequence mask from the xored bytes. Let's do that now.
    for byte in bytes.iter_mut()
    {
        *byte^=sequence;
        sequence = sequence.wrapping_add(1);
    }
    //Create the "crypted" blocks. Each block should be xored with the "key" now.
    for chunk in bytes.chunks_mut(key_size as usize){
        blocks.push(chunk.iter_mut().map(|x| *x).collect::<Vec<u8>>());
    }
    
    let mut ordered: Vec<Vec<(u8, u32)>> = Vec::new();
    
    //We need to get the top 3 occuring 
    for i in 0..x{
        let mut map: HashMap<u8, u32> = HashMap::new();
        for j in 0..y{
            let block = &blocks[j];
            let key = block[i];
            let counter = map.entry(key).or_insert(0);
            *counter += 1;
        }
        let mut count_vec: Vec<(u8, u32)> = map.iter().map(|(x, y)| (*x, *y)).collect();
        count_vec.sort_by(|a, b| b.1.cmp(&a.1));
        ordered.push(count_vec);
    }
    let mut most_frequent_decrypted_as: Vec<Vec<u8>> = Vec::new();
    //We are assuming that either space or e are the most frequent chars. So let's see if that holds true. 
    for i in 0..x{
        let mut top_decrypted: Vec<u8> = Vec::new();
        for j in 0..depth{
            let column_pairs = &ordered[i];
            let crypted = column_pairs[j].0;
            let decrypted = crypted ^ frequent_char;
            top_decrypted.push(decrypted);
        }
        most_frequent_decrypted_as.push(top_decrypted);
    }
    let product = cartesian_product(&most_frequent_decrypted_as[0..]);
    let mut ret: Vec<String> = Vec::new(); 
    for entry in product{
        let check = checksum(&entry[0..]);
        if check == file_checksum {
            if let Ok(a_str) = std::str::from_utf8(&entry){
                ret.push(a_str.to_owned())
            }
        }
    }
    
    Ok(ret)
}

fn get_file_bytes(file_path: &str) -> io::Result<Vec<u8>> {
    let mut f = File::open(file_path)?;
    let mut buffer = Vec::new(); 
    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn write_file_bytes(file_path: &str, bytes: &[u8]) -> io::Result<()>{
    println!("Writing {} bytes to file: {}", bytes.len(), file_path);
     let mut f = File::create(file_path)?;
     f.write_all(bytes)?;
     f.flush()?;
     Ok(())
}

fn crypt(data: &[u8], key_bytes: &[u8]) -> Vec<u8>
{
    let mut ret: Vec<u8> = Vec::new();
    let mut sequence: u8 = (key_bytes.len() + 1) as u8;
    let mut i = 0;
    let modulo = key_bytes.len();
    for cipher_byte in data{
        let index = i%modulo;

        let key_byte = key_bytes[index];
        let cb = *cipher_byte;
        let decrypted = cb ^ sequence ^ key_byte;
        ret.push(decrypted);
        i += 1;
        sequence = sequence.wrapping_add(1);
    }
    ret
}

fn checksum(key_bytes: &[u8]) -> u16
{
    let mut checksum: u16 = 0;
    for byte in key_bytes{
        //Create a temp variable to hold the byte. 
        let mut tmp: u16 = (*byte).into();
        //Shift the byte to the high bits.
        tmp <<= 8;
        //Rotate the checksum 1 bit right. The zero case wont matter.
        checksum = checksum.rotate_right(1);    
        //Xor tmp with the checksum.
        checksum ^= tmp;
    } 
    checksum
}
//Borrowed these two functions: https://gist.github.com/kylewlacy/115965b40e02a3325558. Thanks Kyle!
pub fn partial_cartesian<T: Clone>(a: Vec<Vec<T>>, b: &[T]) -> Vec<Vec<T>> {
    a.into_iter().flat_map(|xs| {
        b.iter().cloned().map(|y| {
            let mut vec = xs.clone();
            vec.push(y);
            vec
        }).collect::<Vec<_>>()
    }).collect()
}
pub fn cartesian_product<U: Clone, T: Clone>(lists: &[U]) -> Vec<Vec<T>>
where
    U: AsRef<[T]>,
    T: Clone
{
    match lists.split_first() {
        Some((first, rest)) => {
            let init: Vec<Vec<T>> = first.as_ref().iter().cloned().map(|n| vec![n]).collect();

            rest.iter().map(|x| x.as_ref().clone()).fold(init, |vec, list| {
                partial_cartesian(vec, list)
            })
        },
        None => {
            vec![]
        }
    }
}
