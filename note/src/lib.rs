use core::option::Option;
use select::document::Document;
use select::node::{Children, Node};
use select::predicate::{Class, Name, Or};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub blocks: Vec<Block>,
    // 可以加入其他元資料，例如標題、作者等
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    pub line_type: String,
    pub attributes: Option<Attributes>,
    pub children: Vec<InlineNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LawCard {
    pub chapter: String,
    pub num: String,
    pub lines: Vec<Line>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Paragraph {
        attributes: Option<Attributes>,
        children: Vec<InlineNode>,
    },
    CustomCard {
        card_type: String,
        data: Option<serde_json::Value>,
    },
    H2 {
        attributes: Option<Attributes>,
        children: Vec<InlineNode>,
    },
    H3 {
        attributes: Option<Attributes>,
        children: Vec<InlineNode>,
    },
    BlockQuote {
        attributes: Option<Attributes>,
        children: Vec<InlineNode>,
    },
    Figure {
        attributes: Option<Attributes>,
        children: Vec<InlineNode>,
    },
    Table {
        attributes: Option<Attributes>,
        children: Vec<InlineNode>,
    }, // 你可以根據需要擴展其他區塊類型
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attributes {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub src: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InlineNode {
    Text {
        text: String,
        attributes: Option<Attributes>,
    },
    Span {
        children: Vec<InlineNode>,
        attributes: Option<Attributes>,
    },
    Img {
        attributes: Option<Attributes>,
    },
    Strong {
        children: Vec<InlineNode>,
        attributes: Option<Attributes>,
    }, // 若有其他行內型態，可擴充此 enum
    P {
        children: Vec<InlineNode>,
        attributes: Option<Attributes>,
    },
}

// 僅保留 chapter 與 num 的 law_card 資料結構
#[derive(Debug, Serialize, Deserialize)]
pub struct LawCardData {
    pub chapter: String,
    pub num: String,
}

fn parse_inline_nodes(node: &Node) -> Vec<InlineNode> {
    let mut nodes = Vec::new();

    for child in node.children() {
        // 若為文字節點，直接取得文字
        if let Some(text) = child.as_text() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                nodes.push(InlineNode::Text {
                    text: trimmed.to_string(),
                    attributes: None,
                });
            }
        } else if let Some(tag) = child.name() {
            // 根據不同標籤建立對應的 InlineNode 變體
            match tag {
                "span" => {
                    // 遞迴處理 span 內部的所有子節點
                    let children = parse_inline_nodes(&child);
                    let attr = Attributes {
                        id: child.attr("id").map(|s| s.to_string()),
                        class: child.attr("class").map(|s| s.to_string()),
                        style: child.attr("style").map(|s| s.to_string()),
                        src: None,
                        width: None,
                        height: None,
                    };
                    nodes.push(InlineNode::Span {
                        children,
                        attributes: Some(attr),
                    });
                }
                "strong" => {
                    // 處理 <strong> 內部的內容，同樣可以包含其他 inline 節點
                    let children = parse_inline_nodes(&child);
                    let attr = Attributes {
                        id: child.attr("id").map(|s| s.to_string()),
                        class: child.attr("class").map(|s| s.to_string()),
                        style: child.attr("style").map(|s| s.to_string()),
                        src: None,
                        width: None,
                        height: None,
                    };
                    nodes.push(InlineNode::Strong {
                        children,
                        attributes: Some(attr),
                    });
                }
                "img" => {
                    let attr = Attributes {
                        id: child.attr("id").map(|s| s.to_string()),
                        class: child.attr("class").map(|s| s.to_string()),
                        style: child.attr("style").map(|s| s.to_string()),
                        src: child.attr("src").map(|s| s.to_string()),
                        width: child.attr("width").map(|s| s.to_string()),
                        height: child.attr("height").map(|s| s.to_string()),
                    };
                    nodes.push(InlineNode::Img {
                        attributes: Some(attr),
                    });
                }
                "p" => {
                    let children = parse_inline_nodes(&child);
                    let attr = Attributes {
                        id: child.attr("id").map(|s| s.to_string()),
                        class: child.attr("class").map(|s| s.to_string()),
                        style: child.attr("style").map(|s| s.to_string()),
                        src: None,
                        width: None,
                        height: None,
                    };
                    nodes.push(InlineNode::P {
                        children,
                        attributes: Some(attr),
                    });
                }

                _ => {
                    // 預設情況下直接遞迴處理內部
                    let children = parse_inline_nodes(&child);
                    nodes.extend(children);
                }
            }
        }
    }
    nodes
}

fn parse_law_card_from_node(node: &select::node::Node) -> LawCard {
    let chapter = node
        .find(Class("law-block-chapter"))
        .next()
        .map(|n| n.text())
        .unwrap_or_default();
    let num = node
        .find(Class("law-block-num"))
        .next()
        .map(|n| n.text())
        .unwrap_or_default();
    let mut buffer = Vec::new();

    let lineNode = node.find(Class("law-block-lines")).next().unwrap();

    if let Some(lawblockline) = lineNode.find(Class("law-block-line")).next() {
        let children = parse_inline_nodes(&lawblockline);
        let attributes = Attributes {
            id: None,
            class: lawblockline.attr("class").map(|s| s.to_string()),
            style: lawblockline.attr("style").map(|s| s.to_string()),
            src: None,
            width: None,
            height: None,
        };
        buffer.push(Line {
            line_type: "normal".to_string(),
            attributes: Some(attributes),
            children,
        })
    } else if let Some(lawindent) = lineNode.find(Class("law-indent")).next() {
        let children = parse_inline_nodes(&lawindent);
        let attributes = Attributes {
            id: None,
            class: lawindent.attr("class").map(|s| s.to_string()),
            style: lawindent.attr("style").map(|s| s.to_string()),
            src: None,
            width: None,
            height: None,
        };
        buffer.push(Line {
            line_type: "indent".to_string(),
            attributes: Some(attributes),
            children,
        })
    }

    LawCard {
        chapter,
        num,
        lines: buffer,
    }
}

/// 使用 select crate 解析 HTML，並依照文件中順序建立 Note 結構
pub fn parse_note(html: &str) -> Vec<Block> {
    // 解析整份 HTML 為 Document
    let document = Document::from(html);
    // 綜合選擇器：同時選取 <p> 與具有 law-block class 的元素
    let mut blocks = Vec::new();
    let selector = Or(
        Or(Or(Name("p"), Name("blockquote")), Class("law-block")),
        Or(Name("h2"), Or(Name("h3"), Name("figure"))),
    ); // 依照文件中的出現順序遍歷所有匹配的節點
    for node in document.find(selector) {
        let node_name = node.name();
        // 如果節點是 law-block（或其 class 包含 "law-block"），則處理為 law-card
        if node_name.unwrap() == "div"
            && node
                .attr("class")
                .map(|s| s.contains("law-block"))
                .unwrap_or(false)
        {
            let law_card_data = parse_law_card_from_node(&node);
            blocks.push(Block::CustomCard {
                card_type: "law".to_string(),
                data: Some(serde_json::to_value(law_card_data).unwrap()),
            });
        } else if node_name.unwrap() == "blockquote" {
            let children = parse_inline_nodes(&node);
            let attributes = Attributes {
                id: None,
                class: None,
                style: None,
                src: None,
                width: None,
                height: None,
            };
            blocks.push(Block::BlockQuote {
                attributes: Some(attributes),
                children,
            });
        } else if node_name.unwrap() == "p" {
            // 檢查這個段落是否位於 law-block 內，若是則跳過（避免重複處理）
            let mut in_law_block = false;
            let mut ancestor = node.parent();
            while let Some(parent) = ancestor {
                if parent
                    .attr("class")
                    .map(|s| s.contains("law-block"))
                    .unwrap_or(false)
                    || parent.is(Name("blockquote"))
                {
                    in_law_block = true;
                    break;
                }
                ancestor = parent.parent();
            }
            if in_law_block {
                continue;
            }
            // 處理一般段落：取得文字並建立 InlineNode
            let children = parse_inline_nodes(&node);
            let attributes = Attributes {
                id: node.attr("id").map(|s| s.to_string()),
                class: node.attr("class").map(|s| s.to_string()),
                style: node.attr("style").map(|s| s.to_string()),
                src: None,
                width: None,
                height: None,
            };
            blocks.push(Block::Paragraph {
                attributes: Some(attributes),
                children,
            });
        } else if node_name.unwrap() == "figure" {
            let children = parse_inline_nodes(&node);
            let attributes = Attributes {
                id: node.attr("id").map(|s| s.to_string()),
                class: node.attr("class").map(|s| s.to_string()),
                style: node.attr("style").map(|s| s.to_string()),
                src: node.attr("src").map(|s| s.to_string()),
                width: None,
                height: None,
            };
            blocks.push(Block::Figure {
                attributes: Some(attributes),
                children,
            });
        } else if node_name.unwrap() == "h2" {
            let children = parse_inline_nodes(&node);
            let attributes = Attributes {
                id: node.attr("id").map(|s| s.to_string()),
                class: node.attr("class").map(|s| s.to_string()),
                style: node.attr("style").map(|s| s.to_string()),
                src: None,
                width: None,
                height: None,
            };
            blocks.push(Block::H2 {
                attributes: Some(attributes),
                children,
            });
        } else if node_name.unwrap() == "h3" {
            let children = parse_inline_nodes(&node);
            let attributes = Attributes {
                id: node.attr("id").map(|s| s.to_string()),
                class: node.attr("class").map(|s| s.to_string()),
                style: node.attr("style").map(|s| s.to_string()),
                src: None,
                width: None,
                height: None,
            };
            blocks.push(Block::H3 {
                attributes: Some(attributes),
                children,
            });
        }
    }
    blocks
}
