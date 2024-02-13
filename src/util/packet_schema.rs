use std::{
    ffi::{c_uchar, c_uint, c_ushort},
    fs::File,
    io::{BufWriter, Write},
    marker::PhantomData,
    path::Path,
    ptr::null_mut,
};

use serde::{Deserialize, Serialize};

use crate::shroom_ffi::{
    socket::{CInPacket, COutPacket},
    ztl::{zxarr::ZArray, zxstr::ZXString8},
};

pub trait ShroomPacket {
    const DATA_OFFSET: usize;

    fn raw_data(&self) -> &ZArray<u8>;
    fn len(&self) -> usize;

    fn data(&self) -> &[u8] {
        &self.raw_data().data()[Self::DATA_OFFSET..Self::DATA_OFFSET + self.len()]
    }

    fn opcode(&self) -> u16 {
        let data = self.data();
        u16::from_le_bytes(data[..2].try_into().unwrap())
    }

    fn offset(&self) -> usize;
}

impl ShroomPacket for COutPacket {
    const DATA_OFFSET: usize = 0;

    fn raw_data(&self) -> &ZArray<u8> {
        &self.send_buf
    }

    fn len(&self) -> usize {
        self.offset as usize
    }

    fn offset(&self) -> usize {
        self.offset as usize
    }
}

impl ShroomPacket for CInPacket {
    const DATA_OFFSET: usize = 4;

    fn raw_data(&self) -> &ZArray<u8> {
        &self.recv_buf
    }

    fn len(&self) -> usize {
        self.recv_buf.len() - Self::DATA_OFFSET
    }

    fn offset(&self) -> usize {
        self.offset as usize - Self::DATA_OFFSET
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PacketStructTy {
    I8,
    I16,
    I32,
    Buf(u32),
    Str(u32),
}

impl From<c_uchar> for PacketStructTy {
    fn from(_value: c_uchar) -> Self {
        Self::I8
    }
}

impl From<c_ushort> for PacketStructTy {
    fn from(_value: c_ushort) -> Self {
        Self::I16
    }
}

impl From<c_uint> for PacketStructTy {
    fn from(_value: c_uint) -> Self {
        Self::I32
    }
}

impl From<&[u8]> for PacketStructTy {
    fn from(value: &[u8]) -> Self {
        Self::Buf(value.len() as u32)
    }
}

impl From<&ZXString8> for PacketStructTy {
    fn from(value: &ZXString8) -> Self {
        Self::Str(value.len() as u32)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PacketStructElem {
    ret_address: usize,
    ty: PacketStructTy,
    offset: usize,
}

impl PacketStructElem {
    pub fn new<T: Into<PacketStructTy>>(offset: usize, ret_addr: usize, ty: T) -> Self {
        Self {
            ret_address: ret_addr,
            ty: ty.into(),
            offset,
        }
    }

    pub fn byte_len(&self) -> usize {
        match self.ty {
            PacketStructTy::I8 => 1,
            PacketStructTy::I16 => 2,
            PacketStructTy::I32 => 4,
            PacketStructTy::Buf(ln) | PacketStructTy::Str(ln) => ln as usize + 2,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PacketStruct {
    elements: Vec<PacketStructElem>,
    send_ret_addr: Option<usize>,
    exception_ret_addr: Option<usize>,
    last_known_offset: usize,
}

impl PacketStruct {
    pub fn new_send(send_ret_addr: usize) -> Self {
        Self {
            send_ret_addr: Some(send_ret_addr),
            ..Default::default()
        }
    }

    pub fn new_recv() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn handle_gap(&mut self, offset: usize) {
        if offset > self.last_known_offset {
            let gap = offset - self.last_known_offset;
            self.elements.push(PacketStructElem::new(
                self.last_known_offset,
                usize::MAX,
                PacketStructTy::Buf(gap as u32),
            ));
            self.last_known_offset = offset;
        }
    }

    pub fn add_elem(&mut self, elem: PacketStructElem) {
        self.handle_gap(elem.offset);
        self.last_known_offset = elem.offset + elem.byte_len();
        self.elements.push(elem);
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PacketStructLog<'a> {
    strct: PacketStruct,
    data: Option<&'a [u8]>,
}

#[derive(Debug)]
pub struct PacketStructLogger<P> {
    data_ptr: *mut u8,
    data_size_hint: Option<usize>,
    out_file: BufWriter<File>,
    cur: PacketStruct,
    with_data: bool,
    _p: PhantomData<P>,
}

unsafe impl<P> Send for PacketStructLogger<P> {}
unsafe impl<P> Sync for PacketStructLogger<P> {}



impl<P: ShroomPacket> PacketStructLogger<P> {
    pub fn new(path: impl AsRef<Path>, with_data: bool) -> Self {
        let out_file = BufWriter::new(File::create(path).unwrap());

        Self {
            cur: Default::default(),
            data_ptr: null_mut(),
            data_size_hint: None,
            out_file,
            with_data,
            _p: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.data_ptr = null_mut();
        self.data_size_hint = None;
        self.cur = Default::default();
    }

    pub fn set_packet_data(&mut self, pkt_ptr: &P) {
        self.data_ptr = pkt_ptr.raw_data().as_ptr();
        self.data_size_hint = Some(pkt_ptr.len());
    }

    pub fn add_elem(&mut self, elem: PacketStructElem) {
        self.cur.add_elem(elem);
    }

    fn finish_inner(&mut self) {
        let strct = std::mem::replace(&mut self.cur, Default::default());
        self.write_to_file(strct).unwrap();
        self.clear();
    }

    pub fn finish_process(&mut self, p: &P) {
        self.set_packet_data(p);
        self.finish_inner();
    }

    pub fn finish_incomplete(&mut self, exception_ret_addr: usize) {
        self.cur.exception_ret_addr = Some(exception_ret_addr);
        self.finish_inner();
    }

    pub fn finish_send(&mut self, send_ret_addr: usize, p: &P) {
        self.set_packet_data(p);
        self.cur.send_ret_addr = Some(send_ret_addr);
        self.finish_inner();
    }


    pub fn write_to_file(&mut self, strct: PacketStruct) -> anyhow::Result<()> {
        let data: ZArray<c_uchar> = if !self.data_ptr.is_null() {
            unsafe { ZArray::from_ptr(self.data_ptr) }
        } else {
            ZArray::empty()
        };

        let len = self.data_size_hint.unwrap_or_else(|| data.len());
        let data = if self.with_data && data.len() > P::DATA_OFFSET {
            Some(&data.data()[P::DATA_OFFSET..P::DATA_OFFSET + len])
        } else {
            None
        };

        serde_json::to_writer(&mut self.out_file, &PacketStructLog { strct, data })?;
        writeln!(&mut self.out_file, ",")?;
        self.out_file.flush()?;

        Ok(())
    }
}
