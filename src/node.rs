/// Inline 요소 인터페이스
/// Text, CodeSpan, Emphasis, Strong, Link, Image 등
pub trait Inline {}

/// Block 요소 인터페이스 (모든 블록의 기본)
pub trait Block {}

/// Container Block 인터페이스
/// 다른 블록을 children으로 포함할 수 있는 블록
/// Document, Blockquote, List, ListItem
pub trait ContainerBlock: Block {
    fn children(&self) -> &Vec<Node>;
}

/// Leaf Block 인터페이스
/// 다른 블록을 포함할 수 없는 블록
/// ThematicBreak, Heading, CodeBlock, Paragraph
pub trait LeafBlock: Block {}

/// 리스트 타입
#[derive(Debug, Clone, PartialEq)]
pub enum ListType {
    /// Bullet 리스트 (-, +, *)
    Bullet,
    /// Ordered 리스트 (숫자 + 구분자)
    Ordered {
        /// 구분자 ('.' 또는 ')')
        delimiter: char,
    },
}

/// CommonMark 노드
///
/// ## Block 분류
/// - **Container Blocks**: Document, Blockquote, List, ListItem
/// - **Leaf Blocks**: ThematicBreak, Heading, CodeBlock, Paragraph
///
/// ## Inline 분류
/// - Text (향후: CodeSpan, Emphasis, Strong, Link, Image 등)
#[derive(Debug, PartialEq)]
pub enum Node {
    Document { children: Vec<Node> },
    Heading { level: u8, children: Vec<Node> },
    Paragraph { children: Vec<Node> },
    Blockquote { children: Vec<Node> },
    CodeBlock { info: Option<String>, content: String },
    ThematicBreak,
    /// 리스트
    List {
        /// 리스트 타입
        list_type: ListType,
        /// 시작 번호 (Ordered만 의미, Bullet은 1)
        start: usize,
        /// tight list 여부
        tight: bool,
        /// 리스트 아이템들
        children: Vec<Node>,
    },
    /// 리스트 아이템
    ListItem { children: Vec<Node> },
    Text(String),
}

/// Node에 대한 Block trait 구현
/// Container Block과 Leaf Block 모두 Block
impl Block for Node {}

/// Node에 대한 Inline trait 구현
/// Text variant만 해당
impl Inline for Node {}

impl Node {
    /// Container Block인지 확인
    /// Document, Blockquote, List, ListItem
    pub fn is_container_block(&self) -> bool {
        matches!(
            self,
            Node::Document { .. }
                | Node::Blockquote { .. }
                | Node::List { .. }
                | Node::ListItem { .. }
        )
    }

    /// Leaf Block인지 확인
    /// ThematicBreak, Heading, CodeBlock, Paragraph
    pub fn is_leaf_block(&self) -> bool {
        matches!(
            self,
            Node::ThematicBreak
                | Node::Heading { .. }
                | Node::CodeBlock { .. }
                | Node::Paragraph { .. }
        )
    }

    /// Block인지 확인 (Container 또는 Leaf)
    pub fn is_block(&self) -> bool {
        self.is_container_block() || self.is_leaf_block()
    }

    /// Inline인지 확인
    /// Text (향후: CodeSpan, Emphasis, Strong, Link, Image 등)
    pub fn is_inline(&self) -> bool {
        matches!(self, Node::Text(_))
    }
}

impl Node {
    /// List 노드 생성
    /// items: 각 아이템의 줄들, parse_item: 아이템 내용을 블록 노드들로 변환하는 함수
    pub fn build_list<F>(
        list_type: ListType,
        start: usize,
        tight: bool,
        items: Vec<Vec<String>>,
        parse_item: F,
    ) -> Self
    where
        F: Fn(&str) -> Vec<Node>,
    {
        let children: Vec<Node> = items
            .into_iter()
            .map(|item_lines| {
                let text = item_lines.join("\n");
                let parsed_blocks = parse_item(&text);
                Node::ListItem {
                    children: parsed_blocks,
                }
            })
            .collect();

        Node::List {
            list_type,
            start,
            tight,
            children,
        }
    }

    /// Container 노드의 children을 반환
    /// Text 같은 Leaf 노드에서 호출하면 panic
    pub fn children(&self) -> &Vec<Node> {
        match self {
            Node::Document { children } => children,
            Node::Heading { children, .. } => children,
            Node::Paragraph { children } => children,
            Node::Blockquote { children } => children,
            Node::List { children, .. } => children,
            Node::ListItem { children } => children,
            Node::CodeBlock { .. } => panic!("CodeBlock has no children"),
            Node::ThematicBreak => panic!("ThematicBreak has no children"),
            Node::Text(_) => panic!("Text node has no children"),
        }
    }

    /// Text 노드의 문자열을 반환
    pub fn as_text(&self) -> &str {
        match self {
            Node::Text(s) => s,
            _ => panic!("Expected Text node"),
        }
    }

    /// Heading 노드의 레벨을 반환
    pub fn level(&self) -> u8 {
        match self {
            Node::Heading { level, .. } => *level,
            _ => panic!("Expected Heading node"),
        }
    }

    /// Heading 노드인지 확인
    pub fn is_heading(&self) -> bool {
        matches!(self, Node::Heading { .. })
    }

    /// ThematicBreak 노드인지 확인
    pub fn is_thematic_break(&self) -> bool {
        matches!(self, Node::ThematicBreak)
    }

    /// Blockquote 노드인지 확인
    pub fn is_blockquote(&self) -> bool {
        matches!(self, Node::Blockquote { .. })
    }

    /// CodeBlock 노드인지 확인
    pub fn is_code_block(&self) -> bool {
        matches!(self, Node::CodeBlock { .. })
    }

    /// CodeBlock의 info string 반환
    pub fn info(&self) -> Option<&str> {
        match self {
            Node::CodeBlock { info, .. } => info.as_deref(),
            _ => panic!("Expected CodeBlock node"),
        }
    }

    /// CodeBlock의 content 반환
    pub fn content(&self) -> &str {
        match self {
            Node::CodeBlock { content, .. } => content,
            _ => panic!("Expected CodeBlock node"),
        }
    }

    /// List 노드인지 확인
    pub fn is_list(&self) -> bool {
        matches!(self, Node::List { .. })
    }

    /// ListItem 노드인지 확인
    pub fn is_list_item(&self) -> bool {
        matches!(self, Node::ListItem { .. })
    }

    /// List의 타입 반환
    pub fn list_type(&self) -> &ListType {
        match self {
            Node::List { list_type, .. } => list_type,
            _ => panic!("Expected List node"),
        }
    }

    /// List의 시작 번호 반환
    pub fn list_start(&self) -> usize {
        match self {
            Node::List { start, .. } => *start,
            _ => panic!("Expected List node"),
        }
    }

    /// List가 tight인지 반환
    pub fn is_tight(&self) -> bool {
        match self {
            Node::List { tight, .. } => *tight,
            _ => panic!("Expected List node"),
        }
    }

    // 테스트용 빌더 메서드
    #[cfg(test)]
    pub fn text(s: &str) -> Self {
        Node::Text(s.to_string())
    }

    #[cfg(test)]
    pub fn para(children: Vec<Self>) -> Self {
        Node::Paragraph { children }
    }

    #[cfg(test)]
    pub fn item(children: Vec<Self>) -> Self {
        Node::ListItem { children }
    }

    #[cfg(test)]
    pub fn bullet_list(tight: bool, children: Vec<Self>) -> Self {
        Node::List {
            list_type: ListType::Bullet,
            start: 1,
            tight,
            children,
        }
    }

    #[cfg(test)]
    pub fn ordered_list(delimiter: char, start: usize, tight: bool, children: Vec<Self>) -> Self {
        Node::List {
            list_type: ListType::Ordered { delimiter },
            start,
            tight,
            children,
        }
    }

    #[cfg(test)]
    pub fn code_block(info: Option<&str>, content: &str) -> Self {
        Node::CodeBlock {
            info: info.map(|s| s.to_string()),
            content: content.to_string(),
        }
    }

    #[cfg(test)]
    pub fn heading(level: u8, children: Vec<Self>) -> Self {
        Node::Heading { level, children }
    }

    #[cfg(test)]
    pub fn blockquote(children: Vec<Self>) -> Self {
        Node::Blockquote { children }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn create_empty_document() {
        let doc = Node::Document { children: vec![] };
        assert_eq!(doc.children().len(), 0);
    }

    #[test]
    fn create_text_node() {
        let text = Node::Text(String::from("Hello"));
        assert_eq!(text.as_text(), "Hello");
    }

    #[test]
    fn document_with_text() {
        let doc = Node::Document {
            children: vec![Node::Text(String::from("안녕하세요"))],
        };

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].as_text(), "안녕하세요");
    }

    #[test]
    fn document_with_paragraph() {
        let doc = Node::Document {
            children: vec![Node::Paragraph {
                children: vec![Node::Text(String::from("문단 내용"))],
            }],
        };

        assert_eq!(doc.children().len(), 1);
        assert_eq!(doc.children()[0].children()[0].as_text(), "문단 내용");
    }

    #[test]
    fn create_heading_node() {
        let heading = Node::Heading {
            level: 2,
            children: vec![Node::Text(String::from("제목"))],
        };

        assert_eq!(heading.level(), 2);
        assert_eq!(heading.children()[0].as_text(), "제목");
    }

    /// 노드 분류 테스트 (is_container_block, is_leaf_block, is_block, is_inline)
    #[rstest]
    // Container Blocks
    #[case(Node::Document { children: vec![] }, true, false, true, false)]
    #[case(Node::Blockquote { children: vec![] }, true, false, true, false)]
    #[case(Node::List { list_type: ListType::Bullet, start: 1, tight: true, children: vec![] }, true, false, true, false)]
    #[case(Node::ListItem { children: vec![] }, true, false, true, false)]
    // Leaf Blocks
    #[case(Node::ThematicBreak, false, true, true, false)]
    #[case(Node::Heading { level: 1, children: vec![] }, false, true, true, false)]
    #[case(Node::CodeBlock { info: None, content: String::new() }, false, true, true, false)]
    #[case(Node::Paragraph { children: vec![] }, false, true, true, false)]
    // Inline
    #[case(Node::Text(String::new()), false, false, false, true)]
    fn test_node_classification(
        #[case] node: Node,
        #[case] is_container: bool,
        #[case] is_leaf: bool,
        #[case] is_block: bool,
        #[case] is_inline: bool,
    ) {
        assert_eq!(node.is_container_block(), is_container);
        assert_eq!(node.is_leaf_block(), is_leaf);
        assert_eq!(node.is_block(), is_block);
        assert_eq!(node.is_inline(), is_inline);
    }
}
