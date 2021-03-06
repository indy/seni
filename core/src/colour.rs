// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This file is part of Seni

// Seni is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Seni is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// |--------+-----------+-------------+-------------|
// | format | element 0 | element 1   | element 2   |
// |--------+-----------+-------------+-------------|
// | RGB    | R 0..1    | G 0..1      | B 0..1      |
// | HSL    | H 0..360  | S 0..1      | L 0..1      |
// | HSLuv  | H 0..360  | S 0..100    | L 0..100    |
// | LAB    | L 0..100  | A -128..128 | B -128..128 |
// |--------+-----------+-------------+-------------|

use crate::colour_palettes::COLOUR_PALETTES;
use crate::error::{Error, Result};
use crate::keywords::Keyword;
use crate::mathutil;
use crate::packable::{Mule, Packable};
use log::error;
use std;
use std::fmt;
use std::convert::TryFrom;

const COLOUR_UNIT_ANGLE: f64 = 360.0 / 12.0;
const COLOUR_COMPLIMENTARY_ANGLE: f64 = COLOUR_UNIT_ANGLE * 6.0;
const COLOUR_TRIAD_ANGLE: f64 = COLOUR_UNIT_ANGLE * 4.0;

const REF_U: f64 = 0.197_830_006_642_836_807_640;
const REF_V: f64 = 0.468_319_994_938_791_003_700;

//  http://www.brucelindbloom.com/index.html?Equations.html
//  http://www.brucelindbloom.com/index.html?Eqn_RGB_XYZ_Matrix.html

// we're using an sRGB working space with a D65 reference white

// https://uk.mathworks.com/help/images/ref/whitepoint.html
// the D65 whitepoint
const WHITEPOINT_0: f64 = 0.9504;
const WHITEPOINT_1: f64 = 1.0;
const WHITEPOINT_2: f64 = 1.0888;

const CIE_EPSILON: f64 = 0.008_856;
const CIE_KAPPA: f64 = 903.3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ColourFormat {
    Rgb,
    Hsl,
    Hsluv,
    Hsv,
    Lab,
}

// Colour with ColourFormat::Rgb is in sRGB colour space
//
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Colour {
    pub format: ColourFormat,
    pub e0: f32,
    pub e1: f32,
    pub e2: f32,
    pub e3: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ColourPreset {
    Chrome,
    HotlineMiami,
    KnightRider,
    Mars,
    Rainbow,
    Robocop,
    Transformers,
}

#[derive(Clone, Debug)]
pub enum ProcColourType {
    ProceduralColour,
    BezierColour,
}

// ProceduralColour: a = rgb + alpha[0], b = rgb + alpha[0], c = rgb + alpha[0], d = rgb + alpha[0]
// BezierColour: a = rgb + alpha[0], b = rgb + alpha[1], c = rgb + alpha[2], d = rgb + alpha[3]
#[derive(Clone, Debug)]
pub struct ProcColourStateStruct {
    pub proc_colour_type: ProcColourType,
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub c: [f32; 3],
    pub d: [f32; 3],
    pub alpha: [f32; 4],
}

// note: ConvertibleColour's RGB is in linear space, whilst Colour is in sRGB space
#[derive(Debug, Clone, Copy)]
enum ConvertibleColour {
    LinearRGB(f64, f64, f64, f64),
    HSLuv(f64, f64, f64, f64),
    HSL(f64, f64, f64, f64),
    LAB(f64, f64, f64, f64),
    HSV(f64, f64, f64, f64),
    XYZ(f64, f64, f64, f64),
    LUV(f64, f64, f64, f64),
    LCH(f64, f64, f64, f64),
}

impl ColourFormat {
    pub fn from_keyword(kw: Keyword) -> Option<ColourFormat> {
        match kw {
            Keyword::Rgb => Some(ColourFormat::Rgb),
            Keyword::Hsl => Some(ColourFormat::Hsl),
            Keyword::Hsluv => Some(ColourFormat::Hsluv),
            Keyword::Hsv => Some(ColourFormat::Hsv),
            Keyword::Lab => Some(ColourFormat::Lab),
            _ => None,
        }
    }
}

impl fmt::Display for ColourFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ColourFormat::Rgb => write!(f, "rgb"),
            ColourFormat::Hsl => write!(f, "hsl"),
            ColourFormat::Hsluv => write!(f, "hsluv"),
            ColourFormat::Hsv => write!(f, "hsv"),
            ColourFormat::Lab => write!(f, "lab"),
        }
    }
}

impl Packable for Colour {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        match self.format {
            ColourFormat::Rgb => Mule::pack_label_sp(cursor, "RGB"),
            ColourFormat::Hsl => Mule::pack_label_sp(cursor, "HSL"),
            ColourFormat::Hsluv => Mule::pack_label_sp(cursor, "HSLUV"),
            ColourFormat::Hsv => Mule::pack_label_sp(cursor, "HSV"),
            ColourFormat::Lab => Mule::pack_label_sp(cursor, "LAB"),
        };

        Mule::pack_f32_sp(cursor, self.e0);
        Mule::pack_f32_sp(cursor, self.e1);
        Mule::pack_f32_sp(cursor, self.e2);
        Mule::pack_f32(cursor, self.e3);

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let mut rem = cursor;

        let format = if rem.starts_with("RGB ") {
            rem = Mule::skip_forward(rem, "RGB ".len());
            ColourFormat::Rgb
        } else if rem.starts_with("HSL ") {
            rem = Mule::skip_forward(rem, "HSL ".len());
            ColourFormat::Hsl
        } else if rem.starts_with("HSLUV ") {
            rem = Mule::skip_forward(rem, "HSLUV ".len());
            ColourFormat::Hsluv
        } else if rem.starts_with("HSV ") {
            rem = Mule::skip_forward(rem, "HSV ".len());
            ColourFormat::Hsv
        } else if rem.starts_with("LAB ") {
            rem = Mule::skip_forward(rem, "LAB ".len());
            ColourFormat::Lab
        } else {
            error!("Colour::unpack invalid format");
            return Err(Error::Packable);
        };

        let (e0, rem) = Mule::unpack_f32_sp(rem)?;
        let (e1, rem) = Mule::unpack_f32_sp(rem)?;
        let (e2, rem) = Mule::unpack_f32_sp(rem)?;
        let (e3, rem) = Mule::unpack_f32(rem)?;

        Ok((Colour::new(format, e0, e1, e2, e3), rem))
    }
}

impl fmt::Display for Colour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.format {
            ColourFormat::Rgb => write!(f, "rgb"),
            ColourFormat::Hsl => write!(f, "hsl"),
            ColourFormat::Hsluv => write!(f, "hsluv"),
            ColourFormat::Hsv => write!(f, "hsv"),
            ColourFormat::Lab => write!(f, "lab"),
        }
    }
}

fn normalised_colour_from_hex_string(hex_component: &str) -> Result<f32> {
    let value = i32::from_str_radix(hex_component, 16)?;
    Ok(value as f32 / 255.0)
}

impl Colour {
    pub fn new(format: ColourFormat, e0: f32, e1: f32, e2: f32, e3: f32) -> Self {
        Colour {
            format,
            e0,
            e1,
            e2,
            e3,
        }
    }

    pub fn clear_colour() -> Self {
        Colour::new(ColourFormat::Rgb, 0.0, 0.0, 0.0, 0.0)
    }

    // return the 5 colour palette at the given index
    pub fn palette(index: usize) -> Result<Vec<Colour>> {
        Ok(vec![
            Colour::from_rgb_hex(COLOUR_PALETTES[index][0])?,
            Colour::from_rgb_hex(COLOUR_PALETTES[index][1])?,
            Colour::from_rgb_hex(COLOUR_PALETTES[index][2])?,
            Colour::from_rgb_hex(COLOUR_PALETTES[index][3])?,
            Colour::from_rgb_hex(COLOUR_PALETTES[index][4])?,
        ])
    }

    // hex in the form: "ff00ff"
    pub fn from_rgb_hex(hex: &str) -> Result<Self> {
        if hex.len() != 6 {
            error!(
                "Colour::from_rgb_hex expects input as 6 hex digits, actual: {}",
                hex
            );
            return Err(Error::Colour);
        }

        Ok(Colour::new(
            ColourFormat::Rgb,
            normalised_colour_from_hex_string(&hex[0..2])?,
            normalised_colour_from_hex_string(&hex[2..4])?,
            normalised_colour_from_hex_string(&hex[4..])?,
            1.0,
        ))
    }

    pub fn convert(&self, format: ColourFormat) -> Result<Colour> {
        if self.format == format {
            Ok(*self)
        } else {
            let convertible_col: ConvertibleColour = self.into();
            colour(convertible_col.clone_as(format))
        }
    }

    pub fn complementary(&self) -> Result<Colour> {
        colour(ConvertibleColour::from(self).complementary())
    }

    pub fn split_complementary(&self) -> Result<(Colour, Colour)> {
        colour_pair(ConvertibleColour::from(self).split_complementary())
    }

    pub fn analagous(&self) -> Result<(Colour, Colour)> {
        colour_pair(ConvertibleColour::from(self).analagous())
    }

    pub fn triad(&self) -> Result<(Colour, Colour)> {
        colour_pair(ConvertibleColour::from(self).triad())
    }

    pub fn darken(&self, value: f32) -> Result<Colour> {
        let mut c = colour(ConvertibleColour::from(self).clone_as(ColourFormat::Lab))?;

        c.e0 = mathutil::clamp(c.e0 - value, 0.0, 100.0);
        Ok(c)
    }

    pub fn lighten(&self, value: f32) -> Result<Colour> {
        let mut c = colour(ConvertibleColour::from(self).clone_as(ColourFormat::Lab))?;

        c.e0 = mathutil::clamp(c.e0 + value, 0.0, 100.0);
        Ok(c)
    }
}

fn colour(colour: Result<ConvertibleColour>) -> Result<Colour> {
    Colour::try_from(colour?)
}

fn colour_pair(colours: Result<(ConvertibleColour, ConvertibleColour)>) -> Result<(Colour, Colour)> {
    let (c1, c2) = colours?;
    Ok((Colour::try_from(c1)?, Colour::try_from(c2)?))
}

impl Default for Colour {
    fn default() -> Colour {
        Colour {
            format: ColourFormat::Rgb,
            e0: 0.0,
            e1: 0.0,
            e2: 0.0,
            e3: 1.0,
        }
    }
}

impl ColourPreset {
    pub fn from_keyword(kw: Keyword) -> Option<ColourPreset> {
        match kw {
            Keyword::Chrome => Some(ColourPreset::Chrome),
            Keyword::HotlineMiami => Some(ColourPreset::HotlineMiami),
            Keyword::KnightRider => Some(ColourPreset::KnightRider),
            Keyword::Mars => Some(ColourPreset::Mars),
            Keyword::Rainbow => Some(ColourPreset::Rainbow),
            Keyword::Robocop => Some(ColourPreset::Robocop),
            Keyword::Transformers => Some(ColourPreset::Transformers),
            _ => None,
        }
    }

    pub fn get_preset(self) -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3]) {
        match self {
            ColourPreset::Chrome => (
                [0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [1.0, 1.0, 1.0],
                [0.0, 0.1, 0.2],
            ),
            ColourPreset::HotlineMiami => (
                [0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [2.0, 1.0, 0.0],
                [0.5, 0.2, 0.25],
            ),
            ColourPreset::KnightRider => (
                [0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [1.0, 0.7, 0.4],
                [0.0, 0.15, 0.2],
            ),
            ColourPreset::Mars => (
                [0.8, 0.5, 0.4],
                [0.2, 0.4, 0.2],
                [2.0, 1.0, 1.0],
                [0.0, 0.25, 0.25],
            ),
            ColourPreset::Rainbow => (
                [0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [1.0, 1.0, 1.0],
                [0.0, 3.33, 6.67],
            ),
            ColourPreset::Robocop => (
                [0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [1.0, 1.0, 1.0],
                [0.3, 0.2, 0.2],
            ),
            ColourPreset::Transformers => (
                [0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [1.0, 1.0, 0.5],
                [0.8, 0.9, 0.3],
            ),
        }
    }
}

impl Default for ProcColourStateStruct {
    fn default() -> ProcColourStateStruct {
        ProcColourStateStruct {
            proc_colour_type: ProcColourType::ProceduralColour,
            a: [0.0, 0.0, 0.0],
            b: [0.0, 0.0, 0.0],
            c: [0.0, 0.0, 0.0],
            d: [0.0, 0.0, 0.0],
            alpha: [1.0, 0.0, 0.0, 0.0],
        }
    }
}

impl ProcColourStateStruct {
    pub fn colour(&self, t: f32) -> Colour {
        match self.proc_colour_type {
            ProcColourType::ProceduralColour => Colour::new(
                ColourFormat::Rgb,
                self.a[0] + self.b[0] * (mathutil::TAU * (self.c[0] * t + self.d[0])).cos(),
                self.a[1] + self.b[1] * (mathutil::TAU * (self.c[1] * t + self.d[1])).cos(),
                self.a[2] + self.b[2] * (mathutil::TAU * (self.c[2] * t + self.d[2])).cos(),
                self.alpha[0],
            ),
            ProcColourType::BezierColour => {
                let r = mathutil::bezier_point(self.a[0], self.b[0], self.c[0], self.d[0], t);
                let g = mathutil::bezier_point(self.a[1], self.b[1], self.c[1], self.d[1], t);
                let b = mathutil::bezier_point(self.a[2], self.b[2], self.c[2], self.d[2], t);
                let alpha = mathutil::bezier_point(
                    self.alpha[0],
                    self.alpha[1],
                    self.alpha[2],
                    self.alpha[3],
                    t,
                );
                Colour::new(ColourFormat::Rgb, r, g, b, alpha)
            }
        }
    }
}

impl From<&Colour> for ConvertibleColour {
    fn from(colour: &Colour) -> ConvertibleColour {
        match colour.format {
            ColourFormat::Rgb => {
                // Colour is in sRGB space, ConvertibleColour is in linear space
                ConvertibleColour::LinearRGB(
                    gamma_expansion(f64::from(colour.e0)),
                    gamma_expansion(f64::from(colour.e1)),
                    gamma_expansion(f64::from(colour.e2)),
                    f64::from(colour.e3),
                )
            },
            ColourFormat::Hsluv => ConvertibleColour::HSLuv(
                f64::from(colour.e0),
                f64::from(colour.e1),
                f64::from(colour.e2),
                f64::from(colour.e3),
            ),
            ColourFormat::Hsl => ConvertibleColour::HSL(
                f64::from(colour.e0),
                f64::from(colour.e1),
                f64::from(colour.e2),
                f64::from(colour.e3),
            ),
            ColourFormat::Lab => ConvertibleColour::LAB(
                f64::from(colour.e0),
                f64::from(colour.e1),
                f64::from(colour.e2),
                f64::from(colour.e3),
            ),
            ColourFormat::Hsv => ConvertibleColour::HSV(
                f64::from(colour.e0),
                f64::from(colour.e1),
                f64::from(colour.e2),
                f64::from(colour.e3),
            ),
        }
    }
}

impl TryFrom<ConvertibleColour> for Colour {
    type Error = Error;

    fn try_from(colour: ConvertibleColour) -> Result<Colour> {
        match colour {
            ConvertibleColour::LinearRGB(r, g, b, a) => {
                Ok(Colour::new(
                    ColourFormat::Rgb,
                    gamma_correction(r) as f32,
                    gamma_correction(g) as f32,
                    gamma_correction(b) as f32,
                    a as f32,
                ))
            },
            ConvertibleColour::HSL(h, s, l, a) => Ok(Colour::new(
                ColourFormat::Hsl,
                h as f32,
                s as f32,
                l as f32,
                a as f32,
            )),
            ConvertibleColour::HSLuv(h, s, l, a) => Ok(Colour::new(
                ColourFormat::Hsluv,
                h as f32,
                s as f32,
                l as f32,
                a as f32,
            )),
            ConvertibleColour::HSV(h, s, v, a) => Ok(Colour::new(
                ColourFormat::Hsv,
                h as f32,
                s as f32,
                v as f32,
                a as f32,
            )),
            ConvertibleColour::LAB(l, a, b, al) => Ok(Colour::new(
                ColourFormat::Lab,
                l as f32,
                a as f32,
                b as f32,
                al as f32,
            )),
            _ => {
                error!("try_from ConvertibleColour to Colour");
                Err(Error::Colour)
            }
        }
    }
}

impl ConvertibleColour {
    fn add_angle_to_hsluv(&self, delta: f64) -> Result<ConvertibleColour> {
        // rotate the hue by the given delta
        if let ConvertibleColour::HSLuv(h, s, l, a) = self.clone_as(ColourFormat::Hsluv)? {
            Ok(ConvertibleColour::HSLuv((h + delta) % 360.0, s, l, a))
        } else {
            error!("add_angle_to_hsluv");
            Err(Error::Colour)
        }
    }

    // Return the 2 colours either side of this that are 'ang' degrees away
    //
    fn pair(&self, ang: f64) -> Result<(ConvertibleColour, ConvertibleColour)> {
        let c1 = self.add_angle_to_hsluv(-ang)?;
        let c2 = self.add_angle_to_hsluv(ang)?;
        Ok((c1, c2))
    }

    // Returns the colour at the opposite end of the wheel
    //
    fn complementary(&self) -> Result<ConvertibleColour> {
        let c1 = self.add_angle_to_hsluv(COLOUR_COMPLIMENTARY_ANGLE)?;
        Ok(c1)
    }

    // Returns the 2 colours next to a complementary colour.
    // e.g. if the input colour is at the 12 o'clock position,
    // this will return the 5 o'clock and 7 o'clock colours
    //
    fn split_complementary(&self) -> Result<(ConvertibleColour, ConvertibleColour)> {
        let c = self.add_angle_to_hsluv(COLOUR_COMPLIMENTARY_ANGLE)?;
        c.pair(COLOUR_UNIT_ANGLE)
    }

    // Returns the adjacent colours.
    // e.g. given a colour at 3 o'clock this will return the
    // colours at 2 o'clock and 4 o'clock
    //
    fn analagous(&self) -> Result<(ConvertibleColour, ConvertibleColour)> {
        self.pair(COLOUR_UNIT_ANGLE)
    }

    // Returns the 2 colours that will result in all 3 colours
    // being evenly spaced around the colour wheel.
    // e.g. given 12 o'clock this will return 4 o'clock and 8 o'clock
    //
    fn triad(&self) -> Result<(ConvertibleColour, ConvertibleColour)> {
        self.pair(COLOUR_TRIAD_ANGLE)
    }

    pub fn clone_as(&self, format: ColourFormat) -> Result<ConvertibleColour> {
        match *self {
            ConvertibleColour::HSL(h, s, l, alpha) => match format {
                ColourFormat::Hsl => Ok(ConvertibleColour::HSL(h, s, l, alpha)),
                ColourFormat::Hsluv => hsluv_from_xyz(xyz_from_rgb(rgb_from_hsl(*self)?)?),
                ColourFormat::Hsv => hsv_from_rgb(rgb_from_hsl(*self)?),
                ColourFormat::Lab => lab_from_xyz(xyz_from_rgb(rgb_from_hsl(*self)?)?),
                ColourFormat::Rgb => rgb_from_hsl(*self),
            },
            ConvertibleColour::HSLuv(h, s, l, alpha) => match format {
                ColourFormat::Hsl => hsl_from_rgb(rgb_from_xyz(xyz_from_hsluv(*self)?)?),
                ColourFormat::Hsluv => Ok(ConvertibleColour::HSLuv(h, s, l, alpha)),
                ColourFormat::Hsv => hsv_from_rgb(rgb_from_xyz(xyz_from_hsluv(*self)?)?),
                ColourFormat::Lab => lab_from_xyz(xyz_from_hsluv(*self)?),
                ColourFormat::Rgb => rgb_from_xyz(xyz_from_hsluv(*self)?),
            },
            ConvertibleColour::HSV(h, s, v, alpha) => match format {
                ColourFormat::Hsl => hsl_from_rgb(rgb_from_hsv(*self)?),
                ColourFormat::Hsluv => hsluv_from_xyz(xyz_from_rgb(rgb_from_hsv(*self)?)?),
                ColourFormat::Hsv => Ok(ConvertibleColour::HSV(h, s, v, alpha)),
                ColourFormat::Lab => lab_from_xyz(xyz_from_rgb(rgb_from_hsv(*self)?)?),
                ColourFormat::Rgb => rgb_from_hsv(*self),
            },
            ConvertibleColour::LAB(l, a, b, alpha) => match format {
                ColourFormat::Hsl => hsl_from_rgb(rgb_from_xyz(xyz_from_lab(*self)?)?),
                ColourFormat::Hsluv => hsluv_from_xyz(xyz_from_lab(*self)?),
                ColourFormat::Hsv => hsv_from_rgb(rgb_from_xyz(xyz_from_lab(*self)?)?),
                ColourFormat::Lab => Ok(ConvertibleColour::LAB(l, a, b, alpha)),
                ColourFormat::Rgb => rgb_from_xyz(xyz_from_lab(*self)?),
            },
            ConvertibleColour::LinearRGB(r, g, b, alpha) => match format {
                ColourFormat::Hsl => hsl_from_rgb(*self),
                ColourFormat::Hsluv => hsluv_from_xyz(xyz_from_rgb(*self)?),
                ColourFormat::Hsv => hsv_from_rgb(*self),
                ColourFormat::Lab => lab_from_xyz(xyz_from_rgb(*self)?),
                ColourFormat::Rgb => Ok(ConvertibleColour::LinearRGB(r, g, b, alpha)),
            },
            _ => Err(Error::Colour),
        }
    }
}

fn gamma_expansion(component: f64) -> f64 {
    if component > 0.04045 {
        ((component + 0.055) / 1.055).powf(2.4)
    } else {
        component / 12.92
    }
}

fn gamma_correction(a: f64) -> f64 {
    if a > 0.003_130_8 {
        (1.055 * a.powf(1.0 / 2.4)) - 0.055
    } else {
        a * 12.92
    }
}

fn xyz_from_rgb(rgb: ConvertibleColour) -> Result<ConvertibleColour> {
    match rgb {
        ConvertibleColour::LinearRGB(r, g, b, alpha) => {
            let x = (r * 0.4124) + (g * 0.3576) + (b * 0.1805);
            let y = (r * 0.2126) + (g * 0.7152) + (b * 0.0722);
            let z = (r * 0.0193) + (g * 0.1192) + (b * 0.9505);

            Ok(ConvertibleColour::XYZ(x, y, z, alpha))
        }
        _ => Err(Error::Colour),
    }
}

fn rgb_from_xyz(xyz: ConvertibleColour) -> Result<ConvertibleColour> {
    match xyz {
        ConvertibleColour::XYZ(x, y, z, alpha) => {
            let r = (x * 3.2406) + (y * -1.5372) + (z * -0.4986);
            let g = (x * -0.9689) + (y * 1.8758) + (z * 0.0415);
            let b = (x * 0.0557) + (y * -0.2040) + (z * 1.0570);

            Ok(ConvertibleColour::LinearRGB(r, g, b, alpha))
        }
        _ => Err(Error::Colour),
    }
}

fn axis_to_lab_component(a: f64) -> f64 {
    if a > CIE_EPSILON {
        a.cbrt()
    } else {
        ((CIE_KAPPA * a) + 16.0) / 116.0
    }
}

fn lab_from_xyz(xyz: ConvertibleColour) -> Result<ConvertibleColour> {
    match xyz {
        ConvertibleColour::XYZ(x, y, z, alpha) => {
            let xr = x / WHITEPOINT_0;
            let yr = y / WHITEPOINT_1;
            let zr = z / WHITEPOINT_2;

            let fx = axis_to_lab_component(xr);
            let fy = axis_to_lab_component(yr);
            let fz = axis_to_lab_component(zr);

            let l = (116.0 * fy) - 16.0;
            let a = 500.0 * (fx - fy);
            let b = 200.0 * (fy - fz);

            Ok(ConvertibleColour::LAB(l, a, b, alpha))
        }
        _ => Err(Error::Colour),
    }
}

fn max_channel(r: f64, g: f64, b: f64) -> i32 {
    let hi = if r > g { 0 } else { 1 };
    let hival = if r > g { r } else { g };

    if b > hival {
        2
    } else {
        hi
    }
}

fn fmod(a: f64, b: f64) -> f64 {
    a - b * (a / b).trunc()
}

// http://www.rapidtables.com/convert/color/rgb-to-hsl.htm
fn hue(colour: ConvertibleColour, max_chan: i32, chroma: f64) -> Result<f64> {
    if chroma == 0.0 {
        // return Err(Error::InvalidConvertibleColourHue)
        return Ok(0.0);
    }

    let mut angle: f64;

    match colour {
        ConvertibleColour::LinearRGB(r, g, b, _) => {
            angle = match max_chan {
                0 => fmod((g - b) / chroma, 6.0),
                1 => ((b - r) / chroma) + 2.0,
                2 => ((r - g) / chroma) + 4.0,
                _ => {
                    error!("invalid colour channel");
                    return Err(Error::Colour);
                }
            }
        }
        _ => return Err(Error::Colour),
    }

    angle *= 60.0;

    while angle < 0.0 {
        angle += 360.0;
    }

    Ok(angle)
}

// http://www.rapidtables.com/convert/color/rgb-to-hsl.htm
fn hsl_from_rgb(colour: ConvertibleColour) -> Result<ConvertibleColour> {
    match colour {
        ConvertibleColour::LinearRGB(r, g, b, alpha) => {
            let min_val = r.min(g).min(b);
            let max_val = r.max(g).max(b);
            let max_ch = max_channel(r, g, b);

            let delta = max_val - min_val;

            let h = hue(colour, max_ch, delta)?;
            let lightness = 0.5 * (min_val + max_val);
            let saturation: f64 = if delta == 0.0 {
                0.0
            } else {
                delta / (1.0 - ((2.0 * lightness) - 1.0).abs())
            };

            Ok(ConvertibleColour::HSL(h, saturation, lightness, alpha))
        }
        _ => Err(Error::Colour),
    }
}

fn hsv_from_rgb(colour: ConvertibleColour) -> Result<ConvertibleColour> {
    match colour {
        ConvertibleColour::LinearRGB(r, g, b, alpha) => {
            let min_val = r.min(g).min(b);
            let max_val = r.max(g).max(b);
            let max_ch = max_channel(r, g, b);

            let chroma = max_val - min_val;
            let h = hue(colour, max_ch, chroma)?;

            // valid_hue: bool = chroma != 0.0;

            let saturation: f64 = if chroma == 0.0 { 0.0 } else { chroma / max_val };

            // TODO: set valid_hue
            // return col.set('valid_hue', valid_hue);

            Ok(ConvertibleColour::HSV(h, saturation, max_val, alpha))
        }
        _ => Err(Error::Colour),
    }
}

fn rgb_from_chm(chroma: f64, h: f64, m: f64, alpha: f64) -> ConvertibleColour {
    // todo: validhue test
    //
    // if (c.get('validHue') === undefined) {
    // return construct(ColourFormat.RGB, [m, m, m, element(c, ALPHA)]);
    //}

    let hprime = h / 60.0;
    let x = chroma * (1.0 - (fmod(hprime, 2.0) - 1.0).abs());
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;

    if hprime < 1.0 {
        r = chroma;
        g = x;
        b = 0.0;
    } else if hprime < 2.0 {
        r = x;
        g = chroma;
        b = 0.0;
    } else if hprime < 3.0 {
        r = 0.0;
        g = chroma;
        b = x;
    } else if hprime < 4.0 {
        r = 0.0;
        g = x;
        b = chroma;
    } else if hprime < 5.0 {
        r = x;
        g = 0.0;
        b = chroma;
    } else if hprime < 6.0 {
        r = chroma;
        g = 0.0;
        b = x;
    }

    ConvertibleColour::LinearRGB(r + m, g + m, b + m, alpha)
}

fn rgb_from_hsl(hsl: ConvertibleColour) -> Result<ConvertibleColour> {
    match hsl {
        ConvertibleColour::HSL(h, s, l, alpha) => {
            let chroma = (1.0 - ((2.0 * l) - 1.0).abs()) * s;
            let m = l - (0.5 * chroma);

            // todo: set validhue
            // f64 col = c.set('validHue', true);

            Ok(rgb_from_chm(chroma, h, m, alpha))
        }
        _ => Err(Error::Colour),
    }
}

fn lab_component_to_axis(l: f64) -> f64 {
    if l.powf(3.0) > CIE_EPSILON {
        l.powf(3.0)
    } else {
        ((116.0 * l) - 16.0) / CIE_KAPPA
    }
}

fn xyz_from_lab(lab: ConvertibleColour) -> Result<ConvertibleColour> {
    match lab {
        ConvertibleColour::LAB(l, a, b, alpha) => {
            let fy = (l + 16.0) / 116.0;
            let fz = fy - (b / 200.0);
            let fx = (a / 500.0) + fy;

            let xr = lab_component_to_axis(fx);
            let mut yr;
            if l > (CIE_EPSILON * CIE_KAPPA) {
                yr = (l + 16.0) / 116.0;
                yr = yr * yr * yr;
            } else {
                yr = l / CIE_KAPPA;
            }
            let zr = lab_component_to_axis(fz);

            Ok(ConvertibleColour::XYZ(
                WHITEPOINT_0 * xr,
                WHITEPOINT_1 * yr,
                WHITEPOINT_2 * zr,
                alpha,
            ))
        }
        _ => Err(Error::Colour),
    }
}

fn rgb_from_hsv(hsv: ConvertibleColour) -> Result<ConvertibleColour> {
    match hsv {
        ConvertibleColour::HSV(h, s, v, alpha) => {
            let chroma = v * s;
            let m = v - chroma;

            Ok(rgb_from_chm(chroma, h, m, alpha))
        }
        _ => Err(Error::Colour),
    }
}

// the luv and hsluv code is based on https://github.com/hsluv/hsluv-c
// which uses the MIT License:

// # The MIT License (MIT)

// Copyright © 2015 Alexei Boronine (original idea, JavaScript implementation)
// Copyright © 2015 Roger Tallada (Obj-C implementation)
// Copyright © 2017 Martin Mitáš (C implementation, based on Obj-C
// implementation)

// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the “Software”),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

#[derive(Debug, Clone, Copy)]
struct Bounds {
    a: f64,
    b: f64,
}

fn get_bounds(l: f64, bounds: &mut [Bounds]) {
    let tl = l + 16.0;
    let sub1 = (tl * tl * tl) / 1_560_896.0;
    let sub2 = if sub1 > CIE_EPSILON {
        sub1
    } else {
        l / CIE_KAPPA
    };

    let mut m = [[0f64; 3]; 3];
    m[0][0] = 3.240_969_941_904_521_343_77;
    m[0][1] = -1.537_383_177_570_093_457_94;
    m[0][2] = -0.498_610_760_293_003_283_66;
    m[1][0] = -0.969_243_636_280_879_826_13;
    m[1][1] = 1.875_967_501_507_720_667_72;
    m[1][2] = 0.041_555_057_407_175_612_47;
    m[2][0] = 0.055_630_079_696_993_608_46;
    m[2][1] = -0.203_976_958_888_976_564_35;
    m[2][2] = 1.056_971_514_242_878_560_72;

    for channel in 0..3 {
        let m1 = m[channel][0];
        let m2 = m[channel][1];
        let m3 = m[channel][2];

        for t in 0..2 {
            let top1 = (284_517.0 * m1 - 94_839.0 * m3) * sub2;
            let top2 = (838_422.0 * m3 + 769_860.0 * m2 + 731_718.0 * m1) * l * sub2
                - 769_860.0 * (t as f64) * l;
            let bottom = (632_260.0 * m3 - 126_452.0 * m2) * sub2 + 126_452.0 * (t as f64);

            bounds[channel * 2 + t].a = top1 / bottom;
            bounds[channel * 2 + t].b = top2 / bottom;
        }
    }
}

fn ray_length_until_intersect(theta: f64, line: &Bounds) -> f64 {
    line.b / (theta.sin() - line.a * theta.cos())
}

fn max_chroma_for_lh(l: f64, h: f64) -> f64 {
    let mut min_len = std::f64::MAX;
    let hrad = h * 0.017_453_292_519_943_295_77; /* (2 * pi / 260) */
    let mut bounds = [Bounds { a: 0.0, b: 0.0 }; 6];

    get_bounds(l, &mut bounds);

    for b in &bounds {
        let l2 = ray_length_until_intersect(hrad, &b);

        if l2 >= 0.0 && l2 < min_len {
            min_len = l2;
        }
    }

    min_len
}

/* http://en.wikipedia.org/wiki/CIELUV
 * In these formulas, Yn refers to the reference white point. We are using
 * illuminant D65, so Yn (see refY in Maxima file) equals 1. The formula is
 * simplified accordingly.
 */
fn y2l(y: f64) -> f64 {
    if y <= CIE_EPSILON {
        y * CIE_KAPPA
    } else {
        116.0 * y.cbrt() - 16.0
    }
}

fn l2y(l: f64) -> f64 {
    if l <= 8.0 {
        l / CIE_KAPPA
    } else {
        let x = (l + 16.0) / 116.0;
        x * x * x
    }
}

fn luv_from_xyz(xyz: ConvertibleColour) -> Result<ConvertibleColour> {
    match xyz {
        ConvertibleColour::XYZ(x, y, z, alpha) => {
            let var_u = (4.0 * x) / (x + (15.0 * y) + (3.0 * z));
            let var_v = (9.0 * y) / (x + (15.0 * y) + (3.0 * z));
            let l = y2l(y);
            let u = 13.0 * l * (var_u - REF_U);
            let v = 13.0 * l * (var_v - REF_V);

            if l < 0.000_000_01 {
                Ok(ConvertibleColour::LUV(l, 0.0, 0.0, alpha))
            } else {
                Ok(ConvertibleColour::LUV(l, u, v, alpha))
            }
        }
        _ => Err(Error::Colour),
    }
}

fn xyz_from_luv(luv: ConvertibleColour) -> Result<ConvertibleColour> {
    match luv {
        ConvertibleColour::LUV(l, u, v, alpha) => {
            if l <= 0.000_000_01 {
                return Ok(ConvertibleColour::XYZ(0.0, 0.0, 0.0, alpha));
            }

            let var_u = u / (13.0 * l) + REF_U;
            let var_v = v / (13.0 * l) + REF_V;
            let y = l2y(l);
            let x = -(9.0 * y * var_u) / ((var_u - 4.0) * var_v - var_u * var_v);
            let z = (9.0 * y - (15.0 * var_v * y) - (var_v * x)) / (3.0 * var_v);

            Ok(ConvertibleColour::XYZ(x, y, z, alpha))
        }
        _ => Err(Error::Colour),
    }
}

fn lch_from_luv(luv: ConvertibleColour) -> Result<ConvertibleColour> {
    match luv {
        ConvertibleColour::LUV(l, u, v, alpha) => {
            let mut h: f64;
            let c = (u * u + v * v).sqrt();

            if c < 0.000_000_01 {
                h = 0.0;
            } else {
                h = v.atan2(u) * 57.295_779_513_082_320_876_80; /* (180 / pi) */
                if h < 0.0 {
                    h += 360.0;
                }
            }

            Ok(ConvertibleColour::LCH(l, c, h, alpha))
        }
        _ => Err(Error::Colour),
    }
}

fn luv_from_lch(lch: ConvertibleColour) -> Result<ConvertibleColour> {
    match lch {
        ConvertibleColour::LCH(l, c, h, alpha) => {
            let hrad = h * 0.017_453_292_519_943_295_77; /* (pi / 180.0) */
            let u = hrad.cos() * c;
            let v = hrad.sin() * c;

            Ok(ConvertibleColour::LUV(l, u, v, alpha))
        }
        _ => Err(Error::Colour),
    }
}

fn lch_from_hsluv(hsluv: ConvertibleColour) -> Result<ConvertibleColour> {
    match hsluv {
        ConvertibleColour::HSLuv(h, s, l, alpha) => {
            let c = if l > 99.999_999_9 || l < 0.000_000_01 {
                0.0
            } else {
                max_chroma_for_lh(l, h) / 100.0 * s
            };

            if s < 0.000_000_01 {
                Ok(ConvertibleColour::LCH(l, c, 0.0, alpha))
            } else {
                Ok(ConvertibleColour::LCH(l, c, h, alpha))
            }
        }
        _ => Err(Error::Colour),
    }
}

fn hsluv_from_lch(lch: ConvertibleColour) -> Result<ConvertibleColour> {
    match lch {
        ConvertibleColour::LCH(l, c, h, alpha) => {
            let s = if l > 99.999_999_9 || l < 0.000_000_01 {
                0.0
            } else {
                c / max_chroma_for_lh(l, h) * 100.0
            };

            if c < 0.000_000_01 {
                Ok(ConvertibleColour::HSLuv(0.0, s, l, alpha))
            } else {
                Ok(ConvertibleColour::HSLuv(h, s, l, alpha))
            }
        }
        _ => Err(Error::Colour),
    }
}

fn xyz_from_hsluv(hsluv: ConvertibleColour) -> Result<ConvertibleColour> {
    xyz_from_luv(luv_from_lch(lch_from_hsluv(hsluv)?)?)
}

fn hsluv_from_xyz(xyz: ConvertibleColour) -> Result<ConvertibleColour> {
    hsluv_from_lch(lch_from_luv(luv_from_xyz(xyz)?)?)
}

#[cfg(test)]
mod tests {

    use super::*;

    const TOLERANCE: f64 = 0.02;

    fn f64_within(tolerance: f64, a: f64, b: f64, msg: &'static str) {
        assert!(
            (a - b).abs() < tolerance,
            format!("{} expected: {}, actual: {}", msg, b, a)
        )
    }

    fn is_format(expected: ColourFormat, actual: ColourFormat) {
        assert!(
            expected == actual,
            format!("expected: {:?}, actual: {:?}", expected, actual)
        )
    }

    fn assert_colour(col: Colour, format: ColourFormat, e0: f32, e1: f32, e2: f32, e3: f32) {
        is_format(col.format, format);
        assert_eq!(col.e0, e0);
        assert_eq!(col.e1, e1);
        assert_eq!(col.e2, e2);
        assert_eq!(col.e3, e3);
    }

    fn assert_hex_colour(hex: &str, format: ColourFormat, e0: f32, e1: f32, e2: f32, e3: f32) {
        if let Ok(col) = Colour::from_rgb_hex(hex) {
            assert_colour(col, format, e0, e1, e2, e3);
        } else {
            assert!(false);
        }
    }

    fn assert_col(
        col: ConvertibleColour,
        format: ColourFormat,
        c0: f64,
        c1: f64,
        c2: f64,
        c3: f64,
    ) {
        match col {
            ConvertibleColour::HSL(h, s, l, alpha) => {
                is_format(format, ColourFormat::Hsl);
                f64_within(TOLERANCE, h, c0, "HSL H");
                f64_within(TOLERANCE, s, c1, "HSL_S");
                f64_within(TOLERANCE, l, c2, "HSL_L");
                f64_within(TOLERANCE, alpha, c3, "HSL_alpha");
            }
            ConvertibleColour::HSLuv(h, s, l, alpha) => {
                is_format(format, ColourFormat::Hsluv);
                f64_within(TOLERANCE, h, c0, "HSLuv H");
                f64_within(TOLERANCE, s, c1, "HSLuv_S");
                f64_within(TOLERANCE, l, c2, "HSLuv_L");
                f64_within(TOLERANCE, alpha, c3, "HSLuv_alpha");
            }
            ConvertibleColour::HSV(h, s, v, alpha) => {
                is_format(format, ColourFormat::Hsv);
                f64_within(TOLERANCE, h, c0, "HSV H");
                f64_within(TOLERANCE, s, c1, "HSV_S");
                f64_within(TOLERANCE, v, c2, "HSV_V");
                f64_within(TOLERANCE, alpha, c3, "HSV_alpha");
            }
            ConvertibleColour::LAB(l, a, b, alpha) => {
                is_format(format, ColourFormat::Lab);
                f64_within(TOLERANCE, l, c0, "LAB_L");
                f64_within(TOLERANCE, a, c1, "LAB_A");
                f64_within(TOLERANCE, b, c2, "LAB_B");
                f64_within(TOLERANCE, alpha, c3, "LAB_alpha");
            }
            ConvertibleColour::LinearRGB(r, g, b, alpha) => {
                is_format(format, ColourFormat::Rgb);
                f64_within(TOLERANCE, r, c0, "RGB R");
                f64_within(TOLERANCE, g, c1, "RGB_G");
                f64_within(TOLERANCE, b, c2, "RGB_B");
                f64_within(TOLERANCE, alpha, c3, "RGB_alpha");
            }
            _ => assert_eq!(true, false),
        }
    }

    fn assert_colour_match(expected: ConvertibleColour, col: ConvertibleColour) {
        match expected {
            ConvertibleColour::HSL(h, s, l, alpha) => {
                assert_col(col, ColourFormat::Hsl, h, s, l, alpha)
            }
            ConvertibleColour::HSLuv(h, s, l, alpha) => {
                assert_col(col, ColourFormat::Hsluv, h, s, l, alpha)
            }
            ConvertibleColour::HSV(h, s, v, alpha) => {
                assert_col(col, ColourFormat::Hsv, h, s, v, alpha)
            }
            ConvertibleColour::LAB(l, a, b, alpha) => {
                assert_col(col, ColourFormat::Lab, l, a, b, alpha)
            }
            ConvertibleColour::LinearRGB(r, g, b, alpha) => {
                assert_col(col, ColourFormat::Rgb, r, g, b, alpha)
            }
            _ => assert_eq!(true, false),
        }
    }

    fn assert_colour_rgb_hsl_match(r: f64, g: f64, b: f64, h: f64, s: f64, l: f64) {
        let rgb = ConvertibleColour::LinearRGB(r, g, b, 1.0);
        let hsl = ConvertibleColour::HSL(h, s, l, 1.0);

        assert_colour_match(rgb, hsl.clone_as(ColourFormat::Rgb).unwrap());
        assert_colour_match(hsl, rgb.clone_as(ColourFormat::Hsl).unwrap());
    }

    #[test]
    fn test_colour() {
        let rgb = ConvertibleColour::LinearRGB(0.2, 0.098, 0.490, 1.0);
        let hsl = ConvertibleColour::HSL(255.61224489795921, 0.6666666666666667, 0.294, 1.0);
        let lab = ConvertibleColour::LAB(45.35732700452649, 35.86129495040863, -46.563641465309246, 1.0);

        assert_colour_match(rgb, rgb.clone_as(ColourFormat::Rgb).unwrap());
        assert_colour_match(rgb, hsl.clone_as(ColourFormat::Rgb).unwrap());
        assert_colour_match(rgb, lab.clone_as(ColourFormat::Rgb).unwrap());

        assert_colour_match(hsl, rgb.clone_as(ColourFormat::Hsl).unwrap());
        assert_colour_match(hsl, hsl.clone_as(ColourFormat::Hsl).unwrap());
        assert_colour_match(hsl, lab.clone_as(ColourFormat::Hsl).unwrap());

        assert_colour_match(lab, rgb.clone_as(ColourFormat::Lab).unwrap());
        assert_colour_match(lab, hsl.clone_as(ColourFormat::Lab).unwrap());
        assert_colour_match(lab, lab.clone_as(ColourFormat::Lab).unwrap());
    }

    #[test]
    fn test_colour_2() {
        let rgb = ConvertibleColour::LinearRGB(0.066666, 0.8, 0.86666666, 1.0);
        let hsluv = ConvertibleColour::HSLuv(198.68061346079247,
                                             90.10860113883753,
                                             84.42735766561451, 1.0);

        assert_colour_match(rgb, hsluv.clone_as(ColourFormat::Rgb).unwrap());
        assert_colour_match(hsluv, rgb.clone_as(ColourFormat::Hsluv).unwrap());
    }

    #[test]
    fn test_colour_3() {
        assert_colour_rgb_hsl_match(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert_colour_rgb_hsl_match(1.0, 1.0, 1.0, 0.0, 0.0, 1.0);
        assert_colour_rgb_hsl_match(1.0, 0.0, 0.0, 0.0, 1.0, 0.5);
        assert_colour_rgb_hsl_match(0.0, 1.0, 0.0, 120.0, 1.0, 0.5);
        assert_colour_rgb_hsl_match(0.0, 0.0, 1.0, 240.0, 1.0, 0.5);
        assert_colour_rgb_hsl_match(1.0, 1.0, 0.0, 60.0, 1.0, 0.5);
        assert_colour_rgb_hsl_match(0.0, 1.0, 1.0, 180.0, 1.0, 0.5);
        assert_colour_rgb_hsl_match(1.0, 0.0, 1.0, 300.0, 1.0, 0.5);
        assert_colour_rgb_hsl_match(0.7529, 0.7529, 0.7529, 0.0, 0.0, 0.75);
        assert_colour_rgb_hsl_match(0.5, 0.5, 0.5, 0.0, 0.0, 0.5);
        assert_colour_rgb_hsl_match(0.5, 0.0, 0.0, 0.0, 1.0, 0.25);
        assert_colour_rgb_hsl_match(0.5, 0.5, 0.0, 60.0, 1.0, 0.25);
        assert_colour_rgb_hsl_match(0.0, 0.5, 0.0, 120.0, 1.0, 0.25);
        assert_colour_rgb_hsl_match(0.5, 0.0, 0.5, 300.0, 1.0, 0.25);
        assert_colour_rgb_hsl_match(0.0, 0.5, 0.5, 180.0, 1.0, 0.25);
        assert_colour_rgb_hsl_match(0.0, 0.0, 0.5, 240.0, 1.0, 0.25);
    }

    #[test]
    fn test_colour_pack() {
        let mut res: String = "".into();
        let col = Colour::new(ColourFormat::Rgb, 1.1, 2.2, 3.3, 4.4);
        col.pack(&mut res).unwrap();
        assert_eq!("RGB 1.1 2.2 3.3 4.4", res);
    }

    #[test]
    fn test_colour_unpack() {
        let (res, _rem) = Colour::unpack("RGB 1.1 2.2 3.3 4.4").unwrap();
        assert_colour(res, ColourFormat::Rgb, 1.1, 2.2, 3.3, 4.4);
    }

    #[test]
    fn test_hex_colour_parsing() {
        assert_hex_colour("ff00ff", ColourFormat::Rgb, 1.0, 0.0, 1.0, 1.0);
    }

    fn assert_equal_f64(a: f64, b: f64) {
        let diff = (a - b).abs();
        let delta: f64 = 0.0001;

        assert!(diff < delta, "a = {}, b = {}", a, b);
    }

    #[test]
    fn test_fmod() {
        assert_equal_f64(3.0, fmod(3.0, 10.0));
        assert_equal_f64(1.0, fmod(10.0, 3.0));

        assert_equal_f64(45.0, fmod(45.0, 360.0));
        assert_equal_f64(0.0, fmod(360.0, 360.0));
        assert_equal_f64(1.0, fmod(361.0, 360.0));
        assert_equal_f64(359.9, fmod(359.9, 360.0));

        assert_equal_f64(0.0, fmod(0.0, 6.0));
        assert_equal_f64(-0.5, fmod(-0.5, 6.0));
        assert_equal_f64(-2.0, fmod(-2.0, 6.0));
        assert_equal_f64(-5.9, fmod(-5.9, 6.0));
        assert_equal_f64(0.0, fmod(-6.0, 6.0));
        assert_equal_f64(-0.1, fmod(-6.1, 6.0));
        assert_equal_f64(-1.0, fmod(-7.0, 6.0));
    }
}
