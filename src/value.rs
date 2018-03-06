use std::ffi::CStr;
use std::iter::{FromIterator, Map};

use error::Error;
use corvus_core::{Block, List as IList, Record as IRecord, Value as IValue, WithError};
use ruby_sys::string; // for low level string reference
use ruru;
use ruru::{AnyObject, Array, Boolean, Fixnum, Float, Hash, NilClass, Object, Proc, RString, Symbol};

#[derive(Debug, Clone, PartialEq)]
pub struct CorvusValue(AnyObject, Option<Block<CorvusValue>>);

impl WithError for CorvusValue {
  type Error = Error;
}

impl CorvusValue {
  fn non_nil<F, T>(&self, f: F) -> Result<T, Error>
  where
    F: FnOnce(AnyObject) -> Result<T, ruru::result::Error>,
  {
    reject_nil(self.0.clone()).and_then(|v| f(v).map_err(Error::Ruru))
  }

  pub fn to_any_object(&self) -> AnyObject {
    self.0.to_any_object()
  }
}

impl From<AnyObject> for CorvusValue {
  fn from(ao: AnyObject) -> Self {
    CorvusValue(ao, None)
  }
}

/*
impl<T> From<T> for CorvusValue
where
  T: Into<AnyObject>,
{
  fn from(v: T) -> CorvusValue {
    CorvusValue(v.into())
  }
}
*/

impl From<bool> for CorvusValue {
  fn from(v: bool) -> CorvusValue {
    CorvusValue(Boolean::new(v).to_any_object(), None)
  }
}

impl From<f64> for CorvusValue {
  fn from(v: f64) -> CorvusValue {
    CorvusValue(Float::new(v).to_any_object(), None)
  }
}

impl From<u64> for CorvusValue {
  fn from(i: u64) -> CorvusValue {
    CorvusValue(Fixnum::new(i as i64).to_any_object(), None)
  }
}

impl From<String> for CorvusValue {
  fn from(s: String) -> CorvusValue {
    CorvusValue(RString::new(s.as_str()).to_any_object(), None)
  }
}

impl From<Vec<CorvusValue>> for CorvusValue {
  fn from(values: Vec<CorvusValue>) -> CorvusValue {
    CorvusValue(
      Array::from_iter(values.into_iter().map(|v| v.0)).to_any_object(),
      None,
    )
  }
}

impl From<Block<CorvusValue>> for CorvusValue {
  fn from(block: Block<CorvusValue>) -> CorvusValue {
    CorvusValue(NilClass::new().to_any_object(), Some(block))
  }
}

impl FromIterator<CorvusValue> for CorvusValue {
  fn from_iter<I>(iterable: I) -> CorvusValue
  where
    I: IntoIterator<Item = CorvusValue>,
  {
    let array: Array = iterable.into_iter().map(|v| v.to_any_object()).collect();
    CorvusValue(array.to_any_object(), None)
  }
}

impl FromIterator<(String, CorvusValue)> for CorvusValue {
  fn from_iter<I>(iterable: I) -> CorvusValue
  where
    I: IntoIterator<Item = (String, CorvusValue)>,
  {
    let mut hash: Hash = Hash::new();
    for (key, val) in iterable {
      hash.store(RString::from(key), val.to_any_object());
    }
    CorvusValue(hash.to_any_object(), None)
  }
}

impl IValue for CorvusValue {
  type List = List;
  type Record = Record;

  fn try_number(&self) -> Result<f64, Error> {
    self.non_nil(|v| v.try_convert_to().map(|f: Float| f.to_f64()))
  }

  fn try_time(&self) -> Result<u64, Error> {
    self.non_nil(|v| v.try_convert_to().map(|i: Fixnum| i.to_i64() as u64))
  }

  fn try_bool(&self) -> Result<bool, Error> {
    self.non_nil(|v| v.try_convert_to().map(|b: Boolean| b.to_bool()))
  }

  fn try_string(&self) -> Result<&str, Error> {
    let rs: RString = self.non_nil(|ao| ao.try_convert_to())?;
    // this code was snuck out of ruru so I can avoid copying string data
    unsafe {
      let cstr = string::rb_string_value_cstr(&rs.value());
      let str = CStr::from_ptr(cstr).to_str().map_err(Error::Utf8Error)?;
      Ok(str)
    }
  }

  fn try_list(&self) -> Result<List, Error> {
    self.non_nil(|v| v.try_convert_to().map(List))
  }

  fn try_record(&self) -> Result<Record, Error> {
    self.non_nil(|v| v.try_convert_to().map(Record))
  }

  fn try_call(&self, args: &[CorvusValue]) -> Result<CorvusValue, Error> {
    match self.1 {
      Some(ref block) => block.call(args),
      None => {
        let rproc: Proc = self.0.try_convert_to()?;
        let args: Vec<_> = args.iter().map(|a| a.to_any_object()).collect();
        reject_nil(rproc.call(Some(&args))).map(CorvusValue::from)
      }
    }
  }
}

#[derive(Debug)]
pub struct List(Array);

impl IntoIterator for List {
  type Item = CorvusValue;
  type IntoIter = Map<<Array as IntoIterator>::IntoIter, fn(AnyObject) -> CorvusValue>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter().map(CorvusValue::from)
  }
}

impl IList<CorvusValue> for List {
  fn len(&self) -> usize {
    self.0.length()
  }

  fn at(&self, key: usize) -> Option<CorvusValue> {
    let v = self.0.at(key as i64);
    if v.is_nil() {
      None
    } else {
      Some(CorvusValue(v, None))
    }
  }
}

#[derive(Debug)]
pub struct Record(AnyObject);

impl Record {
  fn keys(&self) -> Array {
    self
      .0
      .send("keys", None)
      .try_convert_to()
      .unwrap_or_else(|_| Array::new())
  }

  fn at_any_object<T: Object>(&self, key: T) -> Option<CorvusValue> {
    nil_to_none(self.0.send("[]", Some(&[key.to_any_object()]))).map(CorvusValue::from)
  }
}

impl IntoIterator for Record {
  type Item = (String, CorvusValue);
  type IntoIter = RecordIter;

  fn into_iter(self) -> Self::IntoIter {
    RecordIter::from(self)
  }
}

pub struct RecordIter {
  record: Record,
  keys: Array,
  position: usize,
}

impl From<Record> for RecordIter {
  fn from(record: Record) -> Self {
    let keys = record.keys();
    RecordIter {
      record: record,
      keys: keys,
      position: 0,
    }
  }
}

impl Iterator for RecordIter {
  type Item = (String, CorvusValue);

  fn next(&mut self) -> Option<(String, CorvusValue)> {
    if self.position == self.keys.length() {
      return None;
    }
    let key = self.keys.at(self.position as i64);
    self.position += 1;

    let string_key = key
      .try_convert_to::<Symbol>()
      .map(|s| s.to_string())
      .or_else(|_| key.try_convert_to::<RString>().map(|s| s.to_string()));
    match string_key {
      Err(_) => self.next(),
      Ok(key) => self
        .record
        .at(&key)
        .map(|val| (key, val))
        .or_else(|| self.next()),
    }
  }
}

impl IRecord<CorvusValue> for Record {
  fn at(&self, key: &str) -> Option<CorvusValue> {
    self.at_any_object(Symbol::new(key))
  }
}

fn nil_to_none(o: AnyObject) -> Option<AnyObject> {
  if o.is_nil() {
    None
  } else {
    Some(o)
  }
}

fn reject_nil(o: AnyObject) -> Result<AnyObject, Error> {
  nil_to_none(o).ok_or(Error::Nil)
}

/*
fn indifferent_hash_access<T: Object>(hash: Hash, key: &str) -> AnyObject {
  let mut value = hash.at(Symbol::new(key));
  if value.is_nil() {
    value = hash.at(RString::new(key));
  }
  value
}
*/

/*
#[derive(Debug)]
pub struct Block(Proc);

impl Callable<CorvusValue> for Block {
  fn call(&self, args: &[CorvusValue]) -> Result<CorvusValue, Error> {
    let rb_args: Vec<_> = args.iter().map(|x| x.to_any_object()).collect();
    Ok(CorvusValue(self.0.call(Some(&rb_args))))
  }
}

*/
