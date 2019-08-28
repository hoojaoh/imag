//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2019 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

use handlebars::{HelperDef, JsonRender, RenderError, RenderContext, Helper, Output, Context};
use handlebars::Handlebars as Registry;
use serde_json::value::Value;
use ansi_term::Colour;
use ansi_term::Style;

#[derive(Clone, Copy)]
pub struct ColorizeBlackHelper;

impl HelperDef for ColorizeBlackHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        colorize(Colour::Black, h, out)
    }
}

#[derive(Clone, Copy)]
pub struct ColorizeBlueHelper;

impl HelperDef for ColorizeBlueHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        colorize(Colour::Blue, h, out)
    }
}

#[derive(Clone, Copy)]
pub struct ColorizeCyanHelper;

impl HelperDef for ColorizeCyanHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        colorize(Colour::Cyan, h, out)
    }
}

#[derive(Clone, Copy)]
pub struct ColorizeGreenHelper;

impl HelperDef for ColorizeGreenHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        colorize(Colour::Green, h, out)
    }
}

#[derive(Clone, Copy)]
pub struct ColorizePurpleHelper;

impl HelperDef for ColorizePurpleHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        colorize(Colour::Purple, h, out)
    }
}

#[derive(Clone, Copy)]
pub struct ColorizeRedHelper;

impl HelperDef for ColorizeRedHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        colorize(Colour::Red, h, out)
    }
}

#[derive(Clone, Copy)]
pub struct ColorizeWhiteHelper;

impl HelperDef for ColorizeWhiteHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        colorize(Colour::White, h, out)
    }
}

#[derive(Clone, Copy)]
pub struct ColorizeYellowHelper;

impl HelperDef for ColorizeYellowHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        colorize(Colour::Yellow, h, out)
    }
}

#[inline]
fn colorize(color: Colour, h: &Helper, output: &mut dyn Output) -> Result<(), RenderError> {
    let p = h.param(0).ok_or_else(|| RenderError::new("Too few arguments"))?;

    output.write(&format!("{}", color.paint(p.value().render())))?;
    Ok(())
}

#[derive(Clone, Copy)]
pub struct UnderlineHelper;

impl HelperDef for UnderlineHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
            let p = h.param(0).ok_or_else(|| RenderError::new("Too few arguments"))?;
            let s = Style::new().underline();
            out.write(&format!("{}", s.paint(p.value().render())))?;
            Ok(())
        }
}

#[derive(Clone, Copy)]
pub struct BoldHelper;

impl HelperDef for BoldHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
            let p = h.param(0).ok_or_else(|| RenderError::new("Too few arguments"))?;
            let s = Style::new().bold();
            out.write(&format!("{}", s.paint(p.value().render())))?;
            Ok(())
        }
}

#[derive(Clone, Copy)]
pub struct BlinkHelper;

impl HelperDef for BlinkHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
            let p = h.param(0).ok_or_else(|| RenderError::new("Too few arguments"))?;
            let s = Style::new().blink();
            out.write(&format!("{}", s.paint(p.value().render())))?;
            Ok(())
        }
}

#[derive(Clone, Copy)]
pub struct StrikethroughHelper;

impl HelperDef for StrikethroughHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
            let p = h.param(0).ok_or_else(|| RenderError::new("Too few arguments"))?;
            let s = Style::new().strikethrough();
            out.write(&format!("{}", s.paint(p.value().render())))?;
            Ok(())
        }
}

fn param_to_number(idx: usize, h: &Helper) -> Result<u64, RenderError> {
    match *h.param(idx).ok_or_else(|| RenderError::new("Too few arguments"))?.value() {
        Value::Number(ref num) => num.as_u64().ok_or_else(|| RenderError::new("Number cannot be parsed")),
        _ => Err(RenderError::new("Type error: First argument should be a number")),
    }
}

#[derive(Clone, Copy)]
pub struct LeftPadHelper;

impl HelperDef for LeftPadHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        let count = param_to_number(0, h)? as usize;
        let text = h.param(1).ok_or_else(|| RenderError::new("Too few arguments"))?;
        let text = format!("{:>width$}", text.value().render(), width = count);
        out.write(&text)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct RightPadHelper;

impl HelperDef for RightPadHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        let count = param_to_number(0, h)? as usize;
        let text = h.param(1).ok_or_else(|| RenderError::new("Too few arguments"))?;
        let text = format!("{:width$}", text.value().render(), width = count);
        out.write(&text)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct AbbrevHelper;

impl HelperDef for AbbrevHelper {
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'reg, 'rc>, _r: &'reg Registry, _ctx: &'rc Context, _rc: &mut RenderContext<'reg>, out: &mut dyn Output) -> Result<(), RenderError> {
        let count = param_to_number(0, h)? as usize;
        let text = h.param(1).ok_or_else(|| RenderError::new("Too few arguments"))?.value().render();
        out.write(&text.chars().take(count).collect::<String>())?;
        Ok(())
    }
}

pub fn register_all_color_helpers(handlebars: &mut Registry) {
    handlebars.register_helper("black"  , Box::new(ColorizeBlackHelper));
    handlebars.register_helper("blue"   , Box::new(ColorizeBlueHelper));
    handlebars.register_helper("cyan"   , Box::new(ColorizeCyanHelper));
    handlebars.register_helper("green"  , Box::new(ColorizeGreenHelper));
    handlebars.register_helper("purple" , Box::new(ColorizePurpleHelper));
    handlebars.register_helper("red"    , Box::new(ColorizeRedHelper));
    handlebars.register_helper("white"  , Box::new(ColorizeWhiteHelper));
    handlebars.register_helper("yellow" , Box::new(ColorizeYellowHelper));
}

pub fn register_all_format_helpers(handlebars: &mut Registry) {
    handlebars.register_helper("underline"     , Box::new(UnderlineHelper));
    handlebars.register_helper("bold"          , Box::new(BoldHelper));
    handlebars.register_helper("blink"         , Box::new(BlinkHelper));
    handlebars.register_helper("strikethrough" , Box::new(StrikethroughHelper));
    handlebars.register_helper("lpad"          , Box::new(LeftPadHelper));
    handlebars.register_helper("rpad"          , Box::new(RightPadHelper));
    handlebars.register_helper("abbrev"        , Box::new(AbbrevHelper));
}

