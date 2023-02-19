use full_moon::{ast, node::Node, visitors::Visitor};

#[derive(Debug, Clone, Copy)]
struct LoopDepth {
    byte: usize,
    depth: u32,
}

#[derive(Debug)]
pub struct LoopTracker {
    loop_depths: Vec<LoopDepth>,
}

impl LoopTracker {
    pub fn new(ast: &ast::Ast) -> Self {
        let mut visitor = LoopTrackerVisitor {
            loop_depths: Vec::new(),
            depth: 0,
        };

        visitor.visit_ast(ast);

        assert_eq!(
            visitor.depth, 0,
            "Loop depth at the end of a loop tracker should be 0"
        );

        let mut loop_depths = visitor.loop_depths;

        loop_depths.sort_by_cached_key(|loop_depth| loop_depth.byte);

        Self { loop_depths }
    }

    pub fn depth_at_byte(&self, byte: usize) -> u32 {
        match self
            .loop_depths
            .binary_search_by_key(&byte, |loop_depth| loop_depth.byte)
        {
            Ok(index) => self.loop_depths[index].depth,
            Err(index) => {
                if index == 0 {
                    0
                } else {
                    self.loop_depths[index - 1].depth
                }
            }
        }
    }
}

struct LoopTrackerVisitor {
    loop_depths: Vec<LoopDepth>,
    depth: u32,
}

impl LoopTrackerVisitor {
    fn add_loop_depth(&mut self, node: impl Node) {
        self.depth += 1;

        let Some((start, _)) = node.range() else {
            return;
        };

        self.loop_depths.push(LoopDepth {
            byte: start.bytes(),
            depth: self.depth,
        });
    }

    fn remove_loop_depth(&mut self, node: impl Node) {
        self.depth -= 1;

        let Some((_, end)) = node.range() else {
            return;
        };

        self.loop_depths.push(LoopDepth {
            byte: end.bytes(),
            depth: self.depth,
        });
    }
}

impl Visitor for LoopTrackerVisitor {
    fn visit_generic_for(&mut self, node: &ast::GenericFor) {
        self.add_loop_depth(node.block());
    }

    fn visit_generic_for_end(&mut self, node: &ast::GenericFor) {
        self.remove_loop_depth(node);
    }

    fn visit_numeric_for(&mut self, node: &ast::NumericFor) {
        self.add_loop_depth(node.block());
    }

    fn visit_numeric_for_end(&mut self, node: &ast::NumericFor) {
        self.remove_loop_depth(node);
    }

    fn visit_while(&mut self, node: &ast::While) {
        self.add_loop_depth(node.block());
    }

    fn visit_while_end(&mut self, node: &ast::While) {
        self.remove_loop_depth(node);
    }

    fn visit_repeat(&mut self, node: &ast::Repeat) {
        self.add_loop_depth(node.block());
    }

    fn visit_repeat_end(&mut self, node: &ast::Repeat) {
        self.remove_loop_depth(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use regex::Regex;

    fn expected_depths(code: &str) -> Vec<(usize, u32)> {
        let mut depths = Vec::new();

        let regex = Regex::new(r#"expect\((\d+)\)"#).unwrap();

        for regex_match in regex.captures_iter(code) {
            depths.push((
                regex_match.get(0).unwrap().start(),
                regex_match.get(1).unwrap().as_str().parse().unwrap(),
            ));
        }

        depths
    }

    fn test_depths(code: &str) {
        let expected_depths = expected_depths(code);
        assert!(!expected_depths.is_empty());

        let ast = full_moon::parse(code).unwrap();

        let loop_tracker = LoopTracker::new(&ast);

        for (byte, expected_depth) in expected_depths {
            let actual_depth = loop_tracker.depth_at_byte(byte);

            assert_eq!(actual_depth, expected_depth);
        }
    }

    #[test]
    fn loop_tracker() {
        test_depths(
            r#"
            expect(0)

            while true do
                expect(1)

                for i, v in pairs({}) do
                    expect(2)

                    repeat
                        expect(3)
                    until true
                end

                expect(1)
            end

            expect(0)
        "#,
        );
    }
}
