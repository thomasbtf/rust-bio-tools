use log::info;
use std::error::Error;
use std::str;

use rust_htslib::bam;
use rust_htslib::bam::Read;
use rust_htslib::bam::record::CigarString;

use bio::io::fasta;
use bio::alignment::pairwise;
use itertools::Itertools;

pub fn pseudobam_fix_cigars(
    fasta_in: &str,
    bam_in: &str,
    bam_out: &str,
) -> Result<(), Box<dyn Error>> {
    info!("Reading reference from:\n  {}", bam_in);
    let mut fasta_reader = fasta::IndexedReader::from_file(&fasta_in)?;

    info!("Reading pseudobam input file:\n  {}", bam_in);
    let bam_reader = bam::IndexedReader::from_path(&bam_in)?;

    info!("Writing output bam with accurate correct CIGAR strings to:\n  {}", bam_out);
    let bam_writer = bam::Writer::from_path(
        bam_out,
        &bam::Header::from_template(&bam_reader.header()),
        bam::Format::BAM,
    )?;
    let mut bam_buffer = bam::buffer::RecordBuffer::new(bam_reader, false);
    // values as used by bwa, see: http://bio-bwa.sourceforge.net/bwa.shtml
    let mut aligner = pairwise::Aligner::new(-6, -1, |a: u8, b: u8| if a == b {1i32} else {-4i32});
    let mut reference_seq = Vec::new();
    for reference_name in bam_writer.header().target_names() {
        println!("reference_name: {}", str::from_utf8(reference_name)? );
        if let Ok(_) = fasta_reader.fetch_all(str::from_utf8(reference_name)? ) {
            fasta_reader.read(&mut reference_seq)?;
            if let Ok((_, _)) = bam_buffer.fetch(reference_name, 0, reference_seq.len() as u32) {
                for record in bam_buffer.iter_mut().collect_vec() {
                    println!("reference_seq: {:?}", &reference_seq);
                    println!("record sequence: {:?}", record.seq().as_bytes());
                    let mut new_record = record.clone();
                    let alignment = aligner.semiglobal( &record.seq().as_bytes().as_slice(), &reference_seq);
                    println!("alignment: {:?}", alignment);
                    // new function cigar_string_from_alignment_operations()
                    let cigar_string = CigarString(vec![]);
                    new_record.set(record.qname(), Some(&cigar_string), record.seq().encoded, record.qual());
                    //write out new record here
                }
            }
        }
    }

    Ok(())
}