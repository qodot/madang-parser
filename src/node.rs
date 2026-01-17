pub enum Node {
    Document { children: Vec<Node> },
    Heading { level: u8, children: Vec<Node> },
    Paragraph { children: Vec<Node> },
    ThematicBreak,
    Text(String),
}

impl Node {
    /// Container 노드의 children을 반환
    /// Text 같은 Leaf 노드에서 호출하면 panic
    pub fn children(&self) -> &Vec<Node> {
        match self {
            Node::Document { children } => children,
            Node::Heading { children, .. } => children,
            Node::Paragraph { children } => children,
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

    /// ThematicBreak 노드인지 확인
    pub fn is_thematic_break(&self) -> bool {
        matches!(self, Node::ThematicBreak)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
