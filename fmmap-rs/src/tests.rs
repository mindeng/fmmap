#[cfg(feature = "sync")]
mod sync {
    macro_rules! sync_tests {
        ($([$test_fn: ident, $init: block]), +$(,)?) => {
            use std::io::{Read, Seek, SeekFrom, Write};
            use std::sync::atomic::{AtomicUsize, Ordering};
            use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
            use scopeguard::defer;

            const SANITY_TEXT: &'static str = "Hello, sync file!";
            const MODIFIED_SANITY_TEXT: &'static str = "Hello, modified sync file!";

            $(
            #[test]
            fn $test_fn() {
                let mut file = $init;
                assert_eq!(file.as_mut_slice().len(), 0);
                file.truncate(8096).unwrap(); // 1 KB
                let mut writter = file.writer(0).unwrap();
                writter.write_all(SANITY_TEXT.as_bytes()).unwrap();
                writter.seek(SeekFrom::Start(100)).unwrap();
                writter.write_i8(-8).unwrap();
                writter.write_i16::<BigEndian>(-16).unwrap();
                writter.write_i32::<BigEndian>(-32).unwrap();
                writter.write_i64::<BigEndian>(-64).unwrap();
                writter.flush().unwrap();
                writter.seek(SeekFrom::End(0)).unwrap();
                drop(writter);
                let mut reader = file.reader(0).unwrap();
                let mut buf = [0; SANITY_TEXT.len()];
                reader.read_exact(&mut buf).unwrap();
                assert!(buf.eq(SANITY_TEXT.as_bytes()));
                reader.seek(SeekFrom::Start(100)).unwrap();
                assert_eq!(-8, reader.read_i8().unwrap());
                assert_eq!(-16, reader.read_i16::<BigEndian>().unwrap());
                assert_eq!(-32, reader.read_i32::<BigEndian>().unwrap());
                assert_eq!(-64, reader.read_i64::<BigEndian>().unwrap());

                let mut range_writer = file.range_writer(8000, 96).unwrap();
                range_writer.write_u8(8).unwrap();
                range_writer.write_u16::<BigEndian>(16).unwrap();
                range_writer.write_u32::<BigEndian>(32).unwrap();
                range_writer.write_u64::<BigEndian>(64).unwrap();
                range_writer.flush().unwrap();

                let mut range_reader = file.range_reader(8000, 96).unwrap();
                assert_eq!(8, range_reader.read_u8().unwrap());
                assert_eq!(16, range_reader.read_u16::<BigEndian>().unwrap());
                assert_eq!(32, range_reader.read_u32::<BigEndian>().unwrap());
                assert_eq!(64, range_reader.read_u64::<BigEndian>().unwrap());

                file.write_u8(8, 1000).unwrap();
                file.write_u16(16, 1001).unwrap();
                file.write_u32(32, 1003).unwrap();
                file.write_u64(64, 1007).unwrap();
                file.write_u128(128, 1015).unwrap();
                file.write_u16_le(16, 1031).unwrap();
                file.write_u32_le(32, 1033).unwrap();
                file.write_u64_le(64, 1037).unwrap();
                file.write_u128_le(128, 1045).unwrap();
                file.write_usize(64, 1061).unwrap();
                file.write_usize_le(64, 1069).unwrap();

                assert_eq!(8, file.read_u8(1000).unwrap());
                assert_eq!(16, file.read_u16(1001).unwrap());
                assert_eq!(32, file.read_u32(1003).unwrap());
                assert_eq!(64, file.read_u64(1007).unwrap());
                assert_eq!(128, file.read_u128(1015).unwrap());
                assert_eq!(16, file.read_u16_le(1031).unwrap());
                assert_eq!(32, file.read_u32_le(1033).unwrap());
                assert_eq!(64, file.read_u64_le(1037).unwrap());
                assert_eq!(128, file.read_u128_le(1045).unwrap());
                assert_eq!(64, file.read_usize(1061).unwrap());
                assert_eq!(64, file.read_usize_le(1069).unwrap());

                file.write_i8(-8, 2000).unwrap();
                file.write_i16(-16, 2001).unwrap();
                file.write_i32(-32, 2003).unwrap();
                file.write_i64(-64, 2007).unwrap();
                file.write_i128(-128, 2015).unwrap();
                file.write_i16_le(-16, 2031).unwrap();
                file.write_i32_le(-32, 2033).unwrap();
                file.write_i64_le(-64, 2037).unwrap();
                file.write_i128_le(-128, 2045).unwrap();
                file.write_isize(-64, 2061).unwrap();
                file.write_isize_le(-64, 2069).unwrap();

                assert_eq!(-8, file.read_i8(2000).unwrap());
                assert_eq!(-16, file.read_i16(2001).unwrap());
                assert_eq!(-32, file.read_i32(2003).unwrap());
                assert_eq!(-64, file.read_i64(2007).unwrap());
                assert_eq!(-128, file.read_i128(2015).unwrap());
                assert_eq!(-16, file.read_i16_le(2031).unwrap());
                assert_eq!(-32, file.read_i32_le(2033).unwrap());
                assert_eq!(-64, file.read_i64_le(2037).unwrap());
                assert_eq!(-128, file.read_i128_le(2045).unwrap());
                assert_eq!(-64, file.read_isize(2061).unwrap());
                assert_eq!(-64, file.read_isize_le(2069).unwrap());

                file.write_f32(32.0, 3000).unwrap();
                file.write_f32_le(32.0, 3004).unwrap();
                file.write_f64(64.0, 3008).unwrap();
                file.write_f64_le(64.0, 3016).unwrap();
                assert_eq!(32.0, file.read_f32(3000).unwrap());
                assert_eq!(32.0, file.read_f32_le(3004).unwrap());
                assert_eq!(64.0, file.read_f64(3008).unwrap());
                assert_eq!(64.0, file.read_f64_le(3016).unwrap());

                file.zero_range(3000, 3024);

                file.truncate(0).unwrap();
                file.truncate(100).unwrap();

                let st = file.bytes_mut(0, SANITY_TEXT.len()).unwrap();
                st.copy_from_slice(SANITY_TEXT.as_bytes());

                let n = file.write(MODIFIED_SANITY_TEXT.as_bytes(), 0);
                assert_eq!(n, MODIFIED_SANITY_TEXT.len());

                let mst = file.bytes(0, MODIFIED_SANITY_TEXT.len()).unwrap();
                assert_eq!(mst, MODIFIED_SANITY_TEXT.as_bytes());

                let mut vec = vec![0; MODIFIED_SANITY_TEXT.len()];
                let n = file.read(vec.as_mut_slice(), 0);
                assert_eq!(n, MODIFIED_SANITY_TEXT.len());

                let sm = file.slice_mut(MODIFIED_SANITY_TEXT.len(), 4);
                sm.copy_from_slice(&32u32.to_be_bytes());

                let buf = file.slice(MODIFIED_SANITY_TEXT.len(), 4);
                let n = u32::from_be_bytes(buf.try_into().unwrap());
                assert_eq!(n, 32);

                let v = file.copy_all_to_vec();
                assert_eq!(v.len(), 100);
                assert_eq!(&v[..MODIFIED_SANITY_TEXT.len()], MODIFIED_SANITY_TEXT.as_bytes());
                let v = file.copy_range_to_vec(0, MODIFIED_SANITY_TEXT.len());
                assert_eq!(v.as_slice(), MODIFIED_SANITY_TEXT.as_bytes());

                let unique = UNIQUE.fetch_add(1, Ordering::SeqCst);
                let mut pb = std::env::current_dir().unwrap().parent().unwrap().to_path_buf();
                pb.push("scripts");
                pb.push(format!("sync_file_test_all_{}", unique));
                pb.set_extension("mem");

                file.write_all_to_new_file(&pb).unwrap();
                defer!(let _ = std::fs::remove_file(&pb););

                let mut pb1 = std::env::current_dir().unwrap().parent().unwrap().to_path_buf();
                pb1.push("scripts");
                pb1.push(format!("sync_file_test_range_{}", unique));
                pb1.set_extension("mem");
                defer!(let _ = std::fs::remove_file(&pb1););
                file.write_range_to_new_file(&pb1, 0, MODIFIED_SANITY_TEXT.len()).unwrap();

                let mut file = std::fs::File::open(&pb).unwrap();
                assert_eq!(file.metadata().unwrap().len(), 100);
                let mut buf = vec![0; MODIFIED_SANITY_TEXT.len()];
                file.read_exact(&mut buf).unwrap();
                assert_eq!(buf.as_slice(), MODIFIED_SANITY_TEXT.as_bytes());
                drop(file);

                let mut file = std::fs::File::open(&pb1).unwrap();
                assert_eq!(file.metadata().unwrap().len(), MODIFIED_SANITY_TEXT.len() as u64);
                let mut buf = vec![0; MODIFIED_SANITY_TEXT.len()];
                file.read_exact(&mut buf).unwrap();
                assert_eq!(buf.as_slice(), MODIFIED_SANITY_TEXT.as_bytes());
                drop(file);
            }
            )*
        };
    }

    use crate::raw::DiskMmapFileMut;
    use crate::raw::MemoryMmapFileMut;
    use crate::{MmapFileExt, MmapFileMut, MmapFileMutExt};

    const UNIQUE: AtomicUsize = AtomicUsize::new(0);

    sync_tests!(
        [test_memory_file_mut, {
            MemoryMmapFileMut::new("memory.mem")
        }],
        [test_mmap_file_mut, {
            let mut pb = std::env::current_dir()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf();
            pb.push("scripts");
            pb.push("disk");
            pb.set_extension("mem");
            let mut file = MmapFileMut::from(DiskMmapFileMut::create(&pb).unwrap());
            file.set_remove_on_drop(true);
            file
        }],
    );
}

#[cfg(feature = "tokio-async")]
mod axync {
    macro_rules! tokio_async_tests {
        ($([$test_fn: ident, $init: block]), +$(,)?) => {
            use std::io::SeekFrom;
            use std::sync::atomic::Ordering;
            use scopeguard::defer;
            use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

            const SANITY_TEXT: &'static str = "Hello, async file!";
            const MODIFIED_SANITY_TEXT: &'static str = "Hello, modified async file!";

            $(
                #[tokio::test]
                async fn $test_fn() {
                    let mut file = $init;
                    assert_eq!(file.as_mut_slice().len(), 0);
                    file.truncate(8096).await.unwrap(); // 1 KB
                    let mut writter = file.writer(0).unwrap();
                    AsyncWriteExt::write_all(&mut writter, SANITY_TEXT.as_bytes()).await.unwrap();
                    AsyncSeekExt::seek(&mut writter, SeekFrom::Start(100)).await.unwrap();
                    AsyncWriteExt::write_i8(&mut writter, -8).await.unwrap();
                    AsyncWriteExt::write_i16(&mut writter, -16).await.unwrap();
                    AsyncWriteExt::write_i32(&mut writter, -32).await.unwrap();
                    AsyncWriteExt::write_i64(&mut writter, -64).await.unwrap();
                    writter.flush().await.unwrap();
                    writter.seek(SeekFrom::End(0)).await.unwrap();
                    drop(writter);
                    let mut reader = file.reader(0).unwrap();
                    let mut buf = [0; SANITY_TEXT.len()];
                    reader.read_exact(&mut buf).await.unwrap();
                    assert!(buf.eq(SANITY_TEXT.as_bytes()));
                    AsyncSeekExt::seek(&mut reader, SeekFrom::Start(100)).await.unwrap();
                    assert_eq!(-8, AsyncReadExt::read_i8(&mut reader).await.unwrap());
                    assert_eq!(-16, AsyncReadExt::read_i16(&mut reader).await.unwrap());
                    assert_eq!(-32, AsyncReadExt::read_i32(&mut reader).await.unwrap());
                    assert_eq!(-64, AsyncReadExt::read_i64(&mut reader).await.unwrap());

                    let mut range_writer = file.range_writer(8000, 96).unwrap();
                    AsyncWriteExt::write_u8(&mut range_writer, 8).await.unwrap();
                    AsyncWriteExt::write_u16(&mut range_writer, 16).await.unwrap();
                    AsyncWriteExt::write_u32(&mut range_writer, 32).await.unwrap();
                    AsyncWriteExt::write_u64(&mut range_writer, 64).await.unwrap();
                    range_writer.flush().await.unwrap();

                    let mut range_reader = file.range_reader(8000, 96).unwrap();
                    assert_eq!(8, AsyncReadExt::read_u8(&mut range_reader).await.unwrap());
                    assert_eq!(16, AsyncReadExt::read_u16(&mut range_reader).await.unwrap());
                    assert_eq!(32, AsyncReadExt::read_u32(&mut range_reader).await.unwrap());
                    assert_eq!(64, AsyncReadExt::read_u64(&mut range_reader).await.unwrap());

                    file.write_u8(8, 1000).unwrap();
                    file.write_u16(16, 1001).unwrap();
                    file.write_u32(32, 1003).unwrap();
                    file.write_u64(64, 1007).unwrap();
                    file.write_u128(128, 1015).unwrap();
                    file.write_u16_le(16, 1031).unwrap();
                    file.write_u32_le(32, 1033).unwrap();
                    file.write_u64_le(64, 1037).unwrap();
                    file.write_u128_le(128, 1045).unwrap();
                    file.write_usize(64, 1061).unwrap();
                    file.write_usize_le(64, 1069).unwrap();

                    assert_eq!(8, file.read_u8(1000).unwrap());
                    assert_eq!(16, file.read_u16(1001).unwrap());
                    assert_eq!(32, file.read_u32(1003).unwrap());
                    assert_eq!(64, file.read_u64(1007).unwrap());
                    assert_eq!(128, file.read_u128(1015).unwrap());
                    assert_eq!(16, file.read_u16_le(1031).unwrap());
                    assert_eq!(32, file.read_u32_le(1033).unwrap());
                    assert_eq!(64, file.read_u64_le(1037).unwrap());
                    assert_eq!(128, file.read_u128_le(1045).unwrap());
                    assert_eq!(64, file.read_usize(1061).unwrap());
                    assert_eq!(64, file.read_usize_le(1069).unwrap());

                    file.write_i8(-8, 2000).unwrap();
                    file.write_i16(-16, 2001).unwrap();
                    file.write_i32(-32, 2003).unwrap();
                    file.write_i64(-64, 2007).unwrap();
                    file.write_i128(-128, 2015).unwrap();
                    file.write_i16_le(-16, 2031).unwrap();
                    file.write_i32_le(-32, 2033).unwrap();
                    file.write_i64_le(-64, 2037).unwrap();
                    file.write_i128_le(-128, 2045).unwrap();
                    file.write_isize(-64, 2061).unwrap();
                    file.write_isize_le(-64, 2069).unwrap();

                    assert_eq!(-8, file.read_i8(2000).unwrap());
                    assert_eq!(-16, file.read_i16(2001).unwrap());
                    assert_eq!(-32, file.read_i32(2003).unwrap());
                    assert_eq!(-64, file.read_i64(2007).unwrap());
                    assert_eq!(-128, file.read_i128(2015).unwrap());
                    assert_eq!(-16, file.read_i16_le(2031).unwrap());
                    assert_eq!(-32, file.read_i32_le(2033).unwrap());
                    assert_eq!(-64, file.read_i64_le(2037).unwrap());
                    assert_eq!(-128, file.read_i128_le(2045).unwrap());
                    assert_eq!(-64, file.read_isize(2061).unwrap());
                    assert_eq!(-64, file.read_isize_le(2069).unwrap());

                    file.write_f32(32.0, 3000).unwrap();
                    file.write_f32_le(32.0, 3004).unwrap();
                    file.write_f64(64.0, 3008).unwrap();
                    file.write_f64_le(64.0, 3016).unwrap();
                    assert_eq!(32.0, file.read_f32(3000).unwrap());
                    assert_eq!(32.0, file.read_f32_le(3004).unwrap());
                    assert_eq!(64.0, file.read_f64(3008).unwrap());
                    assert_eq!(64.0, file.read_f64_le(3016).unwrap());

                    file.zero_range(3000, 3024);

                    file.truncate(0).await.unwrap();
                    file.truncate(100).await.unwrap();

                    let st = file.bytes_mut(0, SANITY_TEXT.len()).unwrap();
                    st.copy_from_slice(SANITY_TEXT.as_bytes());

                    let n = file.write(MODIFIED_SANITY_TEXT.as_bytes(), 0);
                    assert_eq!(n, MODIFIED_SANITY_TEXT.len());

                    let mst = file.bytes(0, MODIFIED_SANITY_TEXT.len()).unwrap();
                    assert_eq!(mst, MODIFIED_SANITY_TEXT.as_bytes());

                    let mut vec = vec![0; MODIFIED_SANITY_TEXT.len()];
                    let n = file.read(vec.as_mut_slice(), 0);
                    assert_eq!(n, MODIFIED_SANITY_TEXT.len());

                    let sm = file.slice_mut(MODIFIED_SANITY_TEXT.len(), 4);
                    sm.copy_from_slice(&32u32.to_be_bytes());

                    let buf = file.slice(MODIFIED_SANITY_TEXT.len(), 4);
                    let n = u32::from_be_bytes(buf.try_into().unwrap());
                    assert_eq!(n, 32);

                    let v = file.copy_all_to_vec();
                    assert_eq!(v.len(), 100);
                    assert_eq!(&v[..MODIFIED_SANITY_TEXT.len()], MODIFIED_SANITY_TEXT.as_bytes());
                    let v = file.copy_range_to_vec(0, MODIFIED_SANITY_TEXT.len());
                    assert_eq!(v.as_slice(), MODIFIED_SANITY_TEXT.as_bytes());

                    let unique = UNIQUE.fetch_add(1, Ordering::SeqCst);
                    let mut pb = std::env::current_dir().unwrap().parent().unwrap().to_path_buf();
                    pb.push("scripts");
                    pb.push(format!("async_file_test_all_{}", unique));
                    pb.set_extension("mem");

                    file.write_all_to_new_file(&pb).await.unwrap();
                    defer!(let _ = std::fs::remove_file(&pb););

                    let mut pb1 = std::env::current_dir().unwrap().parent().unwrap().to_path_buf();
                    pb1.push("scripts");
                    pb1.push(format!("async_file_test_range_{}", unique));
                    pb1.set_extension("mem");
                    defer!(let _ = std::fs::remove_file(&pb1););
                    file.write_range_to_new_file(&pb1, 0, MODIFIED_SANITY_TEXT.len()).await.unwrap();

                    let mut file = tokio::fs::File::open(&pb).await.unwrap();
                    assert_eq!(file.metadata().await.unwrap().len(), 100);
                    let mut buf = vec![0; MODIFIED_SANITY_TEXT.len()];
                    file.read_exact(&mut buf).await.unwrap();
                    assert_eq!(buf.as_slice(), MODIFIED_SANITY_TEXT.as_bytes());
                    drop(file);

                    let mut file = tokio::fs::File::open(&pb1).await.unwrap();
                    assert_eq!(file.metadata().await.unwrap().len(), MODIFIED_SANITY_TEXT.len() as u64);
                    let mut buf = vec![0; MODIFIED_SANITY_TEXT.len()];
                    file.read_exact(&mut buf).await.unwrap();
                    assert_eq!(buf.as_slice(), MODIFIED_SANITY_TEXT.as_bytes());
                    drop(file);
                }
            )*
        }
    }

    use crate::raw::AsyncDiskMmapFileMut;
    use crate::raw::AsyncMemoryMmapFileMut;
    use crate::{AsyncMmapFileExt, AsyncMmapFileMut, AsyncMmapFileMutExt};
    use std::sync::atomic::AtomicUsize;

    const UNIQUE: AtomicUsize = AtomicUsize::new(0);

    tokio_async_tests!(
        [test_async_memory_file_mut, {
            AsyncMemoryMmapFileMut::new("memory.mem")
        }],
        [test_async_mmap_file_mut, {
            let mut pb = std::env::current_dir()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf();
            pb.push("scripts");
            pb.push("disk");
            pb.set_extension("mem");
            let mut file = AsyncMmapFileMut::from(AsyncDiskMmapFileMut::create(&pb).await.unwrap());
            file.set_remove_on_drop(true);
            file
        }],
    );
}
