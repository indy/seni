// Copyright (C) 2019 Inderjit Gill

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::colour::Colour;
use crate::error::{Error, Result};
use crate::gene::Gene;
use crate::iname::Iname;
use log::error;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NodeLocation {
    pub line: usize,
    pub character: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeGene {
    // todo: the whole idea of having a gene here seems wrong. maybe a unique id that can be a key into a HashMap?
    pub gene: Option<Gene>, // option because we don't know what gene it is at construction time?
    pub parameter_ast: Vec<Node>,
    pub parameter_prefix: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeMeta {
    pub loc: NodeLocation,
    pub gene_info: Option<NodeGene>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    List(NodeMeta, Vec<Node>),
    Vector(NodeMeta, Vec<Node>),
    Float(NodeMeta, f32, String),
    FromName(NodeMeta, String, Iname),
    Name(NodeMeta, String, Iname),
    Label(NodeMeta, String, Iname),
    String(NodeMeta, String, Iname),
    Tilde(NodeMeta),
    Whitespace(NodeMeta, String),
    Comment(NodeMeta, String),
}

impl NodeMeta {
    pub fn new_with_gene(&self, gene: Gene) -> Self {
        NodeMeta {
            loc: self.loc,
            gene_info: Some(NodeGene {
                gene: Some(gene),
                parameter_ast: Vec::new(),
                parameter_prefix: Vec::new(),
            }),
        }
    }
}

impl NodeLocation {
    fn error_here(&self, msg: &str) {
        error!("[{}:{}]: {}", self.line, self.character, msg);
    }
}

impl Node {
    pub fn error_here(&self, msg: &str) {
        match self {
            Node::List(meta, _) => meta.loc.error_here(msg),
            Node::Vector(meta, _) => meta.loc.error_here(msg),
            Node::Float(meta, _, _) => meta.loc.error_here(msg),
            Node::FromName(meta, _, _) => meta.loc.error_here(msg),
            Node::Name(meta, _, _) => meta.loc.error_here(msg),
            Node::Label(meta, _, _) => meta.loc.error_here(msg),
            Node::String(meta, _, _) => meta.loc.error_here(msg),
            Node::Tilde(meta) => meta.loc.error_here(msg),
            Node::Whitespace(meta, _) => meta.loc.error_here(msg),
            Node::Comment(meta, _) => meta.loc.error_here(msg),
        }
    }

    pub fn get_float(&self, use_genes: bool) -> Result<f32> {
        if let Node::Float(meta, f, _) = self {
            genetic_f32(use_genes, *f, meta)
        } else {
            error!("Node::get_float expected Node::Float not {:?}", self);
            Err(Error::Node)
        }
    }

    pub fn get_iname(&self, use_genes: bool) -> Result<Iname> {
        match self {
            Node::Name(meta, _text, iname) => genetic_iname(use_genes, *iname, meta, gene_name),
            Node::FromName(meta, _text, iname) => genetic_iname(use_genes, *iname, meta, gene_name),
            Node::String(meta, _text, iname) => genetic_iname(use_genes, *iname, meta, gene_string),
            _ => {
                error!(
                    "Node::get_iname expected Node::Name or Node::String not {:?}",
                    self
                );
                Err(Error::Node)
            }
        }
    }

    pub fn get_label_iname(&self, use_genes: bool) -> Result<Iname> {
        if let Node::Label(meta, _, iname) = self {
            genetic_iname(use_genes, *iname, meta, gene_name_or_int)
        } else {
            error!("Node::get_label_iname expected Node::Label not {:?}", self);
            Err(Error::Node)
        }
    }

    pub fn get_colour(&self, use_genes: bool) -> Result<Colour> {
        if let Node::List(meta, _) = self {
            genetic_colour(use_genes, meta)
        } else {
            error!("Node::get_colour expected Node::List not {:?}", self);
            Err(Error::Node)
        }
    }

    pub fn get_2d(&self, use_genes: bool) -> Result<(f32, f32)> {
        if let Node::Vector(meta, _) = self {
            genetic_v2d(use_genes, meta)
        } else {
            error!("Node::get_2d expected Node::Vector not {:?}", self);
            Err(Error::Node)
        }
    }

    pub fn is_semantic(&self) -> bool {
        match *self {
            Node::Comment(_, _) | Node::Whitespace(_, _) | Node::Tilde(_) => false,
            _ => true,
        }
    }

    pub fn is_tilde(&self) -> bool {
        match *self {
            Node::Tilde(_) => true,
            _ => false,
        }
    }

    pub fn is_comment(&self) -> bool {
        match *self {
            Node::Comment(_, _) => true,
            _ => false,
        }
    }

    pub fn is_whitespace(&self) -> bool {
        match *self {
            Node::Whitespace(_, _) => true,
            _ => false,
        }
    }

    pub fn is_name(&self) -> bool {
        match self {
            Node::Name(_, _, _) => true,
            _ => false,
        }
    }

    pub fn is_name_with_iname(&self, iname: Iname) -> bool {
        match self {
            Node::Name(_, _, name_iname) => *name_iname == iname,
            _ => false,
        }
    }

    pub fn is_alterable(&self) -> bool {
        match self {
            Node::List(meta, _)
            | Node::Vector(meta, _)
            | Node::Float(meta, _, _)
            | Node::FromName(meta, _, _)
            | Node::Name(meta, _, _)
            | Node::Label(meta, _, _)
            | Node::String(meta, _, _)
            | Node::Tilde(meta)
            | Node::Whitespace(meta, _)
            | Node::Comment(meta, _) => meta.gene_info.is_some(),
        }
    }

    pub fn has_gene(&self) -> bool {
        match self {
            Node::List(meta, _)
            | Node::Vector(meta, _)
            | Node::Float(meta, _, _)
            | Node::FromName(meta, _, _)
            | Node::Name(meta, _, _)
            | Node::Label(meta, _, _)
            | Node::Tilde(meta)
            | Node::String(meta, _, _)
            | Node::Whitespace(meta, _)
            | Node::Comment(meta, _) => {
                if let Some(gene_info) = &meta.gene_info {
                    gene_info.gene.is_some()
                } else {
                    false
                }
            }
        }
    }

    pub fn get_location(&self) -> NodeLocation {
        match self {
            Node::List(meta, _)
            | Node::Vector(meta, _)
            | Node::Float(meta, _, _)
            | Node::FromName(meta, _, _)
            | Node::Name(meta, _, _)
            | Node::Label(meta, _, _)
            | Node::String(meta, _, _)
            | Node::Tilde(meta)
            | Node::Whitespace(meta, _)
            | Node::Comment(meta, _) => meta.loc,
        }
    }
}

fn gene_name(gene: &Gene) -> Result<Iname> {
    match gene {
        Gene::Name(i) => Ok(*i),
        _ => {
            error!("gene_name: expected Gene::Name");
            Err(Error::Node)
        }
    }
}

fn gene_string(gene: &Gene) -> Result<Iname> {
    match gene {
        Gene::String(i) => Ok(*i),
        _ => {
            error!("gene_string: expected Gene::String");
            Err(Error::Node)
        }
    }
}

fn gene_name_or_int(gene: &Gene) -> Result<Iname> {
    match gene {
        // todo: what type of gene would a Node::Label have?
        Gene::Int(i) => Ok(Iname::new(*i)),
        Gene::Name(i) => Ok(*i),
        _ => {
            error!("gene_name_or_int: expected Gene::Int or Gene::Name");
            Err(Error::Node)
        }
    }
}

fn genetic_iname(
    use_genes: bool,
    iname: Iname,
    meta: &NodeMeta,
    f: fn(&Gene) -> Result<Iname>,
) -> Result<Iname> {
    if use_genes {
        if let Some(gene_info) = &meta.gene_info {
            if let Some(gene) = &gene_info.gene {
                return f(&gene);
            }
        }
    }

    Ok(iname)
}

fn genetic_colour(use_genes: bool, meta: &NodeMeta) -> Result<Colour> {
    if use_genes {
        if let Some(gene_info) = &meta.gene_info {
            if let Some(gene) = &gene_info.gene {
                match gene {
                    Gene::Colour(col) => return Ok(*col),
                    _ => {
                        error!("genetic_colour incompatible gene");
                        return Err(Error::Node);
                    }
                }
            }
        }
    } else {
        error!("genetic_colour: expected to use gene");
    }
    Err(Error::Node)
}

fn genetic_v2d(use_genes: bool, meta: &NodeMeta) -> Result<(f32, f32)> {
    if use_genes {
        if let Some(gene_info) = &meta.gene_info {
            if let Some(gene) = &gene_info.gene {
                match gene {
                    Gene::V2D(x, y) => return Ok((*x, *y)),
                    _ => {
                        error!("genetic_v2d incompatible gene");
                        return Err(Error::Node);
                    }
                }
            }
        }
    } else {
        error!("genetic_v2d: expected to use gene");
    }
    Err(Error::Node)
}

fn genetic_f32(use_genes: bool, float: f32, meta: &NodeMeta) -> Result<f32> {
    if use_genes {
        if let Some(gene_info) = &meta.gene_info {
            if let Some(gene) = &gene_info.gene {
                match gene {
                    Gene::Float(f) => return Ok(*f),
                    _ => {
                        error!("genetic_f32 incompatible gene");
                        return Err(Error::Node);
                    }
                }
            }
        }
    }

    Ok(float)
}
