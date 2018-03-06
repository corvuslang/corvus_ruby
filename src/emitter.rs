use std::io;
use std::string::FromUtf8Error;
use corvus_core::{Apply, Prim, Scope, Syntax};

#[derive(Debug)]
pub enum EmitError {
  IO(io::Error),
  Encoding(FromUtf8Error),
}

pub fn emit(stx: &Syntax) -> Result<String, EmitError> {
  let mut buf = vec![];
  {
    let mut emitter = RubyEmitter::new(&mut buf);
    emitter.emit_method_definition(stx).map_err(EmitError::IO)?;
  };
  String::from_utf8(buf).map_err(EmitError::Encoding)
}

struct RubyEmitter<'writer, W: io::Write + 'writer> {
  scope: Scope<()>,
  writer: &'writer mut W,
}

impl<'writer, W: io::Write> RubyEmitter<'writer, W> {
  fn new(writer: &'writer mut W) -> Self {
    RubyEmitter {
      scope: Scope::new(),
      writer: writer,
    }
  }

  fn emit_method_definition(&mut self, stx: &Syntax) -> io::Result<()> {
    write!(self.writer, "def call(**εε)\n")?;
    self.emit(stx)?;
    write!(self.writer, "\nend")
  }

  fn emit(&mut self, stx: &Syntax) -> io::Result<()> {
    match *stx {
      Syntax::Atom(ref prim) => match *prim {
        Prim::Boolean(v) => write!(self.writer, "{:?}", v),
        Prim::String(ref s) => write!(self.writer, "{:?}", s),
        Prim::Number(n) => write!(self.writer, "{:?}", n),
        Prim::Time(t) => write!(self.writer, "Time.at({:?})", t),
        Prim::Money(ref c, ref a) => write!(self.writer, "Money.parse({:?} + {:?}.to_s)", c, a),
      },
      Syntax::Block(ref arg_names, ref body) => {
        write!(self.writer, "Proc.new{{")?;
        if arg_names.len() > 0 {
          let mut block_scope = self.scope.new_child();
          write!(self.writer, "|")?;
          let mut first = true;
          for arg_name in arg_names.iter() {
            if first {
              first = false;
            } else {
              write!(self.writer, ",")?;
            }
            write!(self.writer, "ε_{}", arg_name)?;
            block_scope.insert(arg_name.clone(), ());
          }
          write!(self.writer, "|")?;
          let old_scope = self.scope.clone();
          self.scope = block_scope;
          self.emit(body)?;
          self.scope = old_scope;
        } else {
          self.emit(body)?;
        }
        write!(self.writer, "}}")
      }
      Syntax::Variable(ref path) => {
        match self.scope.get(&path[0]) {
          None => write!(self.writer, "εε[:{}]", &path[0])?,
          _ => write!(self.writer, "ε_{}", &path[0])?,
        }
        if path.len() > 1 {
          write!(self.writer, ".{}", &path[1..].join("."))?;
        }
        Ok(())
      }
      Syntax::List(ref items) => {
        write!(self.writer, "[")?;
        let mut first = true;
        for item in items.iter() {
          if first {
            first = false;
          } else {
            write!(self.writer, ", ")?;
          }
          self.emit(item)?;
        }
        write!(self.writer, "]")
      }
      Syntax::Record(ref entries) => {
        write!(self.writer, "{{")?;
        let mut first = true;
        for &(ref k, ref v) in entries.iter() {
          if first {
            first = false;
          } else {
            write!(self.writer, ",")?;
          }
          write!(self.writer, "{}:", k)?;
          self.emit(v)?;
        }
        write!(self.writer, "}}")
      }
      Syntax::Apply(ref apply) => {
        if apply
          .iter()
          .next()
          .map(|&(ref n, _)| n == "calc")
          .unwrap_or(false)
        {
          // oh yes
          return self.emit_math(apply);
        }
        write!(self.writer, "self.corvus_call(")?;
        let mut first = true;
        for &(ref name, ref value) in apply.iter() {
          if first {
            first = false;
          } else {
            write!(self.writer, ",")?;
          }
          write!(self.writer, ":{},", name)?;
          self.emit(value)?;
        }
        write!(self.writer, ")")
      }
    }
  }

  fn emit_math(&mut self, apply: &Apply<Syntax>) -> io::Result<()> {
    // calc: 1 plus: 2 times: 3
    // (((1+2)*3)-5)
    for _ in apply.iter() {
      write!(self.writer, "(")?;
    }
    for &(ref op, ref val) in apply.iter() {
      match op.as_str() {
        "plus" => write!(self.writer, "+")?,
        "subtract" => write!(self.writer, "-")?,
        "times" => write!(self.writer, "*")?,
        "dividedBy" => write!(self.writer, "/")?,
        _ => (),
      }
      self.emit(val)?;
      write!(self.writer, ")")?;
    }
    Ok(())
  }
}

impl<'writer, W: io::Write> io::Write for RubyEmitter<'writer, W> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.writer.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.writer.flush()
  }
}

#[cfg(test)]
mod tests {
  macro_rules! ruby_emit_eq {
    ($corvus_src:expr, $ruby_output:expr) => {{
      use super::RubyEmitter;
      use value::CorvusValue;
      use corvus_core::{parse, Namespace, ParseRule};

      let mut buf = vec![];
      {
        let mut emitter = RubyEmitter::new(&mut buf);
        let ns: Namespace<CorvusValue> = Namespace::new_with_prelude().unwrap();
        let stx = parse(&ns, ParseRule::term, $corvus_src).unwrap();
        emitter.emit(&stx).unwrap();
      }
      let output_string = String::from_utf8(buf).unwrap();
      assert_eq!(output_string.as_str(), $ruby_output);
    }};
  }

  #[test]
  fn test_emit_string() {
    ruby_emit_eq!("\"hello world\"", "\"hello world\"");
  }

  #[test]
  fn test_emit_list() {
    ruby_emit_eq!("[ 1 2 3 ]", "[1.0, 2.0, 3.0]");
  }

  #[test]
  fn test_emit_record() {
    ruby_emit_eq!("[ a=1 b=2 c=3 ]", "{a:1.0,b:2.0,c:3.0}");
  }

  #[test]
  fn test_emit_block() {
    ruby_emit_eq!("{ x y => [x y] }", "Proc.new{|ε_x,ε_y|[ε_x, ε_y]}");
  }

  #[test]
  fn test_emit_apply() {
    ruby_emit_eq!(
      "countFrom: 1 to: 10",
      "self.corvus_call(:countFrom,1.0,:to,10.0)"
    );
  }

  #[test]
  fn test_emit_larger() {
    let src = "each: offices do: { office => [
        name = get_name: office
        employees = each: office.employees do: { e => get_name: e }
    ] }";
    let out = "self.corvus_call(:each,εε[:offices],:do,Proc.new{|ε_office|{name:self.corvus_call(:get_name,ε_office),employees:self.corvus_call(:each,ε_office.employees,:do,Proc.new{|ε_e|self.corvus_call(:get_name,ε_e)})}})";
    ruby_emit_eq!(src, out);
  }
}
