use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use ark_bn254::Bn254;
use ark_ec::short_weierstrass::{Affine, SWCurveConfig};
use ark_groth16::{prepare_verifying_key, ProvingKey};

use rayon::prelude::*;

use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Validate,
};

fn serialize_affine_list<P: SWCurveConfig>(
    input: &[Affine<P>],
    mut writer: impl Write,
) -> Result<(), SerializationError> {
    let serialize = |x: &Affine<P>| {
        let mut raw = Vec::with_capacity(128);
        x.serialize_uncompressed(&mut raw).unwrap();
        raw
    };
    let output: Vec<u8> = input
        .par_iter()
        .with_min_len(16384)
        .flat_map(serialize)
        .collect();
    writer.write_all(&(output.len() as u64).to_le_bytes())?;
    writer.write_all(&output[..])?;
    Ok(())
}

fn deserialize_affine_list<P: SWCurveConfig, const CHECK: bool>(
    mut reader: impl Read,
) -> Result<Vec<Affine<P>>, SerializationError> {
    let item_length = Affine::<P>::default().serialized_size(Compress::No);

    let check = if CHECK { Validate::Yes } else { Validate::No };

    let mut size = [0u8; 8];
    reader.read_exact(&mut size)?;
    let size = u64::from_le_bytes(size) as usize;

    let mut raw = vec![0u8; size];
    reader.read_exact(&mut raw)?;

    let deserialize = |x: &[u8]| Affine::<P>::deserialize_with_mode(x, Compress::No, check);
    raw.par_chunks_exact(item_length)
        .with_min_len(16384)
        .map(deserialize)
        .collect()
}

pub fn save_key(
    current_dir: &PathBuf,
    name: &str,
    key: ProvingKey<Bn254>,
) -> Result<(), SerializationError> {
    let file_name = current_dir.join(format!("{}.pk", name));
    let mut writer = BufWriter::new(File::create(file_name)?);
    key.vk.serialize_uncompressed(&mut writer)?;
    key.beta_g1.serialize_uncompressed(&mut writer)?;
    key.delta_g1.serialize_uncompressed(&mut writer)?;
    serialize_affine_list(&key.a_query, &mut writer)?;
    serialize_affine_list(&key.b_g1_query, &mut writer)?;
    serialize_affine_list(&key.b_g2_query, &mut writer)?;
    serialize_affine_list(&key.h_query, &mut writer)?;
    serialize_affine_list(&key.l_query, &mut writer)?;
    writer.flush()?;

    let vk = prepare_verifying_key(&key.vk);
    let file_name = current_dir.join(format!("{}.vk", name));
    let mut writer = File::create(file_name)?;
    vk.serialize_uncompressed(&mut writer)?;

    Ok(())
}

pub fn load_proving_key<const CHECK: bool>(
    current_dir: &PathBuf,
    name: &str,
) -> Result<ProvingKey<Bn254>, SerializationError> {
    let file_name = current_dir.join(format!("{}.pk", name));
    let mut reader = BufReader::new(File::open(file_name).unwrap());

    let check = if CHECK { Validate::Yes } else { Validate::No };

    Ok(ProvingKey {
        vk: CanonicalDeserialize::deserialize_with_mode(&mut reader, Compress::No, check)?,
        beta_g1: CanonicalDeserialize::deserialize_with_mode(&mut reader, Compress::No, check)?,
        delta_g1: CanonicalDeserialize::deserialize_with_mode(&mut reader, Compress::No, check)?,
        a_query: deserialize_affine_list::<_, CHECK>(&mut reader)?,
        b_g1_query: deserialize_affine_list::<_, CHECK>(&mut reader)?,
        b_g2_query: deserialize_affine_list::<_, CHECK>(&mut reader)?,
        h_query: deserialize_affine_list::<_, CHECK>(&mut reader)?,
        l_query: deserialize_affine_list::<_, CHECK>(&mut reader)?,
    })
}
