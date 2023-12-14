
mod linker;

use std::fs;
use std::io;
use std::io::Write;
use std::process::exit;
use clap::Parser;

use object::{write, Object, ObjectKind, File};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    input_files: Vec<String>,

    #[arg(short, long, default_value_t = String::from("a.out"))]
    output: String,
}

fn get_input_file_contents(input_files: &Vec<String>) -> Vec<Vec<u8>>
{
    let mut file_content_buf: Vec<Vec<u8>> = vec![];
    for file_name in input_files 
    {
        let file_contents = fs::read(file_name)
            .expect("Could not read file");
        file_content_buf.push(file_contents);
    }
    return file_content_buf;
}

fn parse_input_files(file_content_buf: &Vec<Vec<u8>>) -> Vec<File>
{
    let mut obj_files = vec![];
    for file_contents in file_content_buf
    {
        let bytes: &[u8] = file_contents;
        let obj_file = File::parse(bytes)
            .expect("Could not parse input object file");
        obj_files.push(obj_file);
    }
    return obj_files;
}

fn check_object_kinds(obj_files: &Vec<File>, input_files: &Vec<String>) -> u32 
{
    let mut m_invalid_kinds = 0;
    for (i, &ref obj_file) in obj_files.iter().enumerate() {
        let kind = obj_file.kind();
        if kind != ObjectKind::Relocatable {
            let file_name = &input_files[i];
            m_invalid_kinds = m_invalid_kinds + 1;
            println!("Cannot link {:?} Object \"{}\"", kind, file_name);
        }
    }
    return m_invalid_kinds;
}

fn check_object_incompats(obj_files: &Vec<File>, input_files: &Vec<String>) -> (u32,object::Architecture,object::Endianness)
{
    // Get the traits which must be compatible in the first object file
    let mut m_incompat = 0;
    let o = &obj_files[0];
    let o_name = &input_files[0];
    let o_arch = o.architecture();
    let o_endian = if o.is_little_endian() {object::Endianness::Little} else {object::Endianness::Big};
    let o_class = o.is_64();

    for (i, &ref obj_file) in obj_files.iter().enumerate() 
    {
        let curr_name = &input_files[i];

        let curr_arch = obj_file.architecture();
        let curr_endian = if obj_file.is_little_endian() {object::Endianness::Little} else {object::Endianness::Big};
        let curr_class = obj_file.is_64();

        if curr_arch != o_arch {
            m_incompat = m_incompat + 1;
            println!("Incompatible Architectures between input files \"{}\" and \"{}\", ({:?} != {:?})",
                curr_name, o_name, curr_arch, o_arch);
        }
        if curr_endian != o_endian {
            m_incompat = m_incompat + 1;
            println!("Incompatible Endianness between input files \"{}\" and \"{}\", ({:?} != {:?})",
                curr_name, o_name, curr_endian, o_endian);
        }
        if curr_class != o_class {
            m_incompat = m_incompat + 1;
            println!("Incompatible Classes between input files \"{}\" and \"{}\", ({:?} != {:?})",
                curr_name, o_name, curr_class, o_class);
        }
    }
    return (m_incompat,o_arch,o_endian);
}

fn main() {
    let args = Args::parse();

    if args.input_files.len() == 0 {
        println!("No input files provided!");
        exit(1);
    }

    // Read file contents of input files
    let file_content_buf = get_input_file_contents(&args.input_files);

    // Parse input files as object files 
    let obj_files = parse_input_files(&file_content_buf);

    // Validate that all input files are relocatable object files
    let num_invalid = check_object_kinds(&obj_files, &args.input_files);
    if num_invalid > 0 {
        exit(1);
    }
    // Validate that all object files are compatible
    let (num_incompat,arch,endian) = check_object_incompats(&obj_files, &args.input_files);
    if num_incompat > 0 {
        exit(1);
    }

    let mut output_obj = write::Object::new(object::BinaryFormat::Elf,arch,endian); 

    linker::link_into(&mut output_obj, &obj_files);

    let output_bytes = output_obj.write()
        .expect("Error creating output file contents");

    let mut output_file = fs::OpenOptions::new()
        .write(true)
        .read(false)
        .truncate(true)
        .create(true)
        .open(args.output)
        .expect("Could not open or create output file");

    output_file.write_all(&output_bytes)
        .expect("Could not write to output file");

    exit(0);
}

