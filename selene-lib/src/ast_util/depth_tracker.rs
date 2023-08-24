use full_moon::{ast, node::Node, visitors::Visitor};

#[derive(Debug, Clone, Copy)]
struct Depth {
    byte: usize,
    depth: u32,
}

#[derive(Debug)]
pub struct DepthTracker {
    depths: Vec<Depth>,
}

// TODO: Can we merge depth_tracker here?
impl DepthTracker {
    pub fn new(ast: &ast::Ast) -> Self {
        let mut visitor = DepthTrackerVisitor {
            depths: Vec::new(),
            depth: 0,
        };

        visitor.visit_ast(ast);

        assert_eq!(
            visitor.depth, 0,
            "Depth at the end of a depth tracker should be 0"
        );

        let mut depths = visitor.depths;

        depths.sort_by_cached_key(|depth| depth.byte);

        Self { depths }
    }

    pub fn depth_at_byte(&self, byte: usize) -> u32 {
        match self.depths.binary_search_by_key(&byte, |depth| depth.byte) {
            Ok(index) => self.depths[index].depth,
            Err(index) => {
                if index == 0 {
                    0
                } else {
                    self.depths[index - 1].depth
                }
            }
        }
    }
}

struct DepthTrackerVisitor {
    depths: Vec<Depth>,
    depth: u32,
}

impl DepthTrackerVisitor {
    fn add_depth(&mut self, node: impl Node) {
        self.depth += 1;

        let Some((start, _)) = node.range() else {
            return;
        };

        self.depths.push(Depth {
            byte: start.bytes(),
            depth: self.depth,
        });
    }

    fn remove_depth(&mut self, node: impl Node) {
        self.depth -= 1;

        let Some((_, end)) = node.range() else {
            return;
        };

        self.depths.push(Depth {
            byte: end.bytes(),
            depth: self.depth,
        });
    }
}

impl Visitor for DepthTrackerVisitor {
    fn visit_generic_for(&mut self, node: &ast::GenericFor) {
        self.add_depth(node.block());
    }

    fn visit_generic_for_end(&mut self, node: &ast::GenericFor) {
        self.remove_depth(node);
    }

    fn visit_numeric_for(&mut self, node: &ast::NumericFor) {
        self.add_depth(node.block());
    }

    fn visit_numeric_for_end(&mut self, node: &ast::NumericFor) {
        self.remove_depth(node);
    }

    fn visit_while(&mut self, node: &ast::While) {
        self.add_depth(node.block());
    }

    fn visit_while_end(&mut self, node: &ast::While) {
        self.remove_depth(node);
    }

    fn visit_repeat(&mut self, node: &ast::Repeat) {
        self.add_depth(node.block());
    }

    fn visit_repeat_end(&mut self, node: &ast::Repeat) {
        self.remove_depth(node);
    }
    fn visit_function_body(&mut self, node: &ast::FunctionBody) {
        self.add_depth(node.block());
    }

    fn visit_function_body_end(&mut self, node: &ast::FunctionBody) {
        self.remove_depth(node);
    }

    fn visit_do(&mut self, node: &ast::Do) {
        self.add_depth(node.block());
    }

    fn visit_do_end(&mut self, node: &ast::Do) {
        self.remove_depth(node);
    }

    fn visit_if(&mut self, node: &ast::If) {
        self.add_depth(node.block());
    }

    fn visit_if_end(&mut self, node: &ast::If) {
        self.remove_depth(node);
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

        let depth_tracker = DepthTracker::new(&ast);

        for (byte, expected_depth) in expected_depths {
            let actual_depth = depth_tracker.depth_at_byte(byte);

            assert_eq!(actual_depth, expected_depth);
        }
    }

    #[test]
    fn depth_tracker() {
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

        test_depths(
            r#"
            expect(0)

            while true do
                expect(1)

                local function a()
                    expect(2)

                    b(function()
                        expect(3)
                    end, function()
                        expect(3)

                        if true then
                            expect(4)
                        else
                            expect(4)
                        end
                    end)
                end

                expect(1)
            end

            expect(0)
        "#,
        );
    }
}
