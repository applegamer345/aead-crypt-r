mod lib;
#[macro_use]
extern crate arrayref;

use std::intrinsics::breakpoint;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::process::exit;
use std::thread;
use std::io;
use std::fs;
use std::env;
use std::path::PathBuf;
use std::thread::JoinHandle;
use std::thread::sleep;
use std::time;
use colored::Colorize;

const NONCE: [u8;19] = [63, 108, 88, 113, 58, 82, 76, 123, 68, 60, 58, 27, 234, 187, 63, 17, 96, 244, 92];


fn main() {
    // USAGE: ... [dir] [subdirs]
    let args: Vec<String> = env::args().collect();


    let size = &args.len().to_owned();
    if *size as i32 != 4  {
        println!("\nWrong amount of args,\nUSAGE: ... [DIR: PATH] [ALSO-SUBDIRS: 0/1] [ENCRYPT[1]/DECRYPT[0]] [PASSWORD]");
        exit(1);
    }

    let path = args.get(1).unwrap();
    let sub:bool = args.get(2).unwrap().parse::<i32>().unwrap() != 0;
    let enc:bool = args.get(3).unwrap().parse::<i32>().unwrap() != 0;
    // DOSN'T SAVE THE PASSWORD IN PLAIN MEMORY
    // TODO: AS USER INPUT.
    let _password = lib::password_to_key(args.get(4).unwrap().to_string());



    println!("Now deleting {:#?},\ndeletion of all its sub-dirs: {:#?}\n{}",path,sub,"Press Enter to continue.".yellow());
    
    let mut input = String::new();

    // WAITING FOR USER
    let handle = io::stdin();    
    handle.read_line(&mut input).expect("ERROR HAPPEND EXITTING");
    
    // COUNTING TIME
    let now = std::time::Instant::now();
    
    // GETTING ALL THE FILES
    let re = get_files(path,sub);
    println!("{:#?}",re);

    // INTERACT WITH THE FILES
    interact_with_files(re.to_owned().0, enc, _password);

    // STOP THE TIME
    let took = now.elapsed();

    // SUMMERY
    println!("{:?}",re.0.len());
    println!("Found {:#?} files",re.1);
    println!("It took : {:#?}!",took)
    

}

fn interact(path: PathBuf,encrypt: bool, n: u8, key: [u8;32]) {
    if encrypt {
        let nonce = lib::rand_key_nonce().1;

        lib::encrypt_data_file(&path.to_string_lossy().to_string(), &key, &nonce).expect("Couldn't encrypt! -\\");

        println!("[{n}] Encrypted: {:#?}",path);

    } else {
        if path.to_string_lossy().to_string().contains(".enc")  {
            let mut f = fs::File::open(&path).expect("Couldn't Decrypted Data");
            
            let mut buf = vec![0; 19];

            f.read_exact(&mut buf).expect("Couldn't Decrypted Data");
            lib::decrypt_data_file(&path.to_string_lossy().to_string(), &key, array_ref![buf,0,19]).expect("Couldn't decrypt_data! -\\");
            println!("[{n}] Decrypted: {:#?}",path);

        }   else {
            println!("not")
        }
    }
    fs::remove_file(&path).expect(&"Couldn't delete inital file!".red());
    // SOMETIMES IT ISN'T REALY DELETING THEM BUT CLEARS THEM ATLEAST

}

fn get_files(path: &String, sub: bool) -> (Vec<PathBuf>, i32) {
    let mut count: i32 = 0;
    let mut _files: Vec<PathBuf> = vec![]; 
    for a in fs::read_dir(path).unwrap() {
        let a_path = a.unwrap().path();
        //println!("sub");
        if a_path.is_dir() && sub {
            let path_files = get_files(&a_path.to_string_lossy().to_string(),sub);
            for b in path_files.0 {
                _files.push(b);
                //print!("{count}\n");
            }
            count = count + path_files.1
        }
        
        count = count + 1;
        if a_path.is_file() {
            _files.push(a_path)
        }
        //println!("Path: {:?}:{count}", a_path);
    }
    return (_files,count);
}

fn thread_run(list: Vec<PathBuf>,encrypt: bool,n: u8, key: [u8;32]) -> JoinHandle<()> {
    let a = thread::spawn(move|| {
        for file in list {
            interact(file, encrypt, n, key);

            //println!("{n}|{encrypt}|{:#?}",file)
        }
        println!("[{n}] {} ","FINISHED".green());        
    });
    return a;
}

static NUMBER_OF_THREADS: usize = 4;

fn interact_with_files(files_:Vec<PathBuf>,encrypt:bool, key: [u8;32]) {    
    let mut threads: Vec<JoinHandle<()>> = vec![];
    let mut n: u8 = 0;
    //println!("{:?}", files_.chunks(files_.len()/NUMBER_OF_THREADS).into_iter());
    let file_iter;
    if files_.len() as u8 <= 8 {
        file_iter = files_.chunks(files_.len()).into_iter()
    } else {
        file_iter = files_.chunks(files_.len()/NUMBER_OF_THREADS).into_iter()
    }


    for i in file_iter {
        println!("{n}:{:#?}",i);
        n = n +1;
        threads.push(thread_run(i.to_vec(), encrypt,n, key));
    }
    let mut b = false;
    loop {

        sleep(time::Duration::from_millis(500));
        if b {break;}
        let mut a = 0;
        for t in &threads {
            if t.is_finished() {
                b = true;
                
            } else {
                b = false;
            }
            a = a +1;
        }
    }
    println!("{}","FINISHED COMPLETLY!".green());
}