use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct Circuit {
    broadcast: Vec<String>,
    modules: Vec<Module>,
    pulses: (usize, usize),
    steps: usize,
}

impl Circuit {
    fn new(input: &str) -> Self {
        let pulses = (0, 0);
        let mut broadcast = Vec::new();
        let mut modules = Vec::new();

        for line in input.lines().filter(|l| !l.is_empty()) {
            let (pre, post) = line.split_once(" -> ").unwrap();

            if line.starts_with('%') {
                let name = pre.trim().chars().skip(1).collect();
                let to = post.split(',').map(|s| s.trim().to_string()).collect();
                modules.push(Module::FlipFlop(FlipFlop {
                    name,
                    status: false,
                    to,
                }));
            } else if line.starts_with('&') {
                let name = pre.trim().chars().skip(1).collect();
                let to = post.split(',').map(|s| s.trim().to_string()).collect();
                modules.push(Module::Conjunction(Conjunction {
                    name,
                    received: Vec::new(),
                    to,
                }));
            } else if line.starts_with("broadcaster") {
                broadcast = post.split(',').map(|s| s.trim().to_string()).collect();
            } else {
                panic!("invalid input");
            }
        }

        let modules_clone = modules.clone();

        // fill in all the conjunction connections
        for outer_module in modules.iter_mut() {
            match outer_module {
                Module::FlipFlop(_) => continue,
                Module::Conjunction(conj) => {
                    for inner_module in modules_clone.iter() {
                        if inner_module.sends_to(&conj.name) {
                            conj.received.push((inner_module.get_name(), false));
                        }
                    }
                }
            }
        }

        Self {
            broadcast,
            modules,
            pulses,
            steps: 0,
        }
    }

    fn press_button(&mut self) {
        self.steps += 1;

        let mut queue: VecDeque<Pulse> = self
            .broadcast
            .iter()
            .map(|dest| Pulse {
                from: "broadcaster".into(),
                to: dest.clone(),
                high: false,
            })
            .collect();

        self.pulses.0 += 1 + self.broadcast.len();

        while let Some(pulse) = queue.pop_front() {
            if let Some(module) = self.modules.iter_mut().find(|m| m.is_name(&pulse.to)) {
                let debug = module.is_name("vr");
                let pre = module.clone();
                for new_pulse in module.receive_pulse(&pulse) {
                    if debug {
                        match (pre.clone(), &mut *module) {
                            (Module::Conjunction(pre), Module::Conjunction(post)) => {
                                if pre.received != post.received {
                                    dbg!(&self.steps);
                                    dbg!(&post);
                                }
                            }
                            _ => (),
                        }
                    }
                    if new_pulse.high {
                        self.pulses.1 += 1;
                    } else {
                        self.pulses.0 += 1;
                    }
                    queue.push_back(new_pulse);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Pulse {
    from: String,
    to: String,
    high: bool,
}

#[derive(Debug, Clone)]
enum Module {
    FlipFlop(FlipFlop),
    Conjunction(Conjunction),
}

impl Module {
    fn receive_pulse(&mut self, pulse: &Pulse) -> Vec<Pulse> {
        match self {
            Module::FlipFlop(flipflop) => flipflop.receive_pulse(pulse),
            Module::Conjunction(conjunction) => conjunction.receive_pulse(pulse),
        }
    }

    fn is_name(&self, name: &str) -> bool {
        match self {
            Module::FlipFlop(flipflop) => flipflop.name == name,
            Module::Conjunction(conjunction) => conjunction.name == name,
        }
    }

    fn get_name(&self) -> String {
        match self {
            Module::FlipFlop(flipflop) => flipflop.name.clone(),
            Module::Conjunction(conjunction) => conjunction.name.clone(),
        }
    }

    fn sends_to(&self, module_name: &String) -> bool {
        match self {
            Module::FlipFlop(flipflop) => flipflop.to.contains(module_name),
            Module::Conjunction(conjunction) => conjunction.to.contains(module_name),
        }
    }
}

trait SendReceive {
    fn receive_pulse(&mut self, pulse: &Pulse) -> Vec<Pulse>;
}

#[derive(Debug, Clone)]
struct FlipFlop {
    name: String,
    status: bool,
    to: Vec<String>,
}

impl SendReceive for FlipFlop {
    fn receive_pulse(&mut self, pulse: &Pulse) -> Vec<Pulse> {
        if pulse.high {
            return Vec::new();
        }

        self.status = !self.status;
        self.to
            .iter()
            .map(|dest| Pulse {
                from: self.name.clone(),
                to: dest.clone(),
                high: self.status,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
struct Conjunction {
    name: String,
    received: Vec<(String, bool)>,
    to: Vec<String>,
}

impl SendReceive for Conjunction {
    fn receive_pulse(&mut self, pulse: &Pulse) -> Vec<Pulse> {
        // update memory
        if let Some(record) = self
            .received
            .iter_mut()
            .find(|(sender, _high)| *sender == pulse.from)
        {
            record.1 = pulse.high;
        }

        let high = !self.received.iter().all(|(_sender, high)| *high);

        self.to
            .iter()
            .map(|dest| Pulse {
                from: self.name.clone(),
                to: dest.clone(),
                high,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let mut circuit = Circuit::new(
            "broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a",
        );
        for _ in 0..1000 {
            circuit.press_button();
        }
        assert_eq!(circuit.pulses, (8000, 4000));
    }

    #[test]
    fn example2() {
        let mut circuit = Circuit::new(
            "broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output
",
        );

        for _ in 0..1000 {
            circuit.press_button();
        }
        assert_eq!(circuit.pulses, (4250, 2750));
    }

    #[test]
    fn part1() {
        let input = std::fs::read_to_string("input.txt").unwrap();
        let mut circuit = Circuit::new(&input);

        for _ in 0..15000 {
            circuit.press_button();
        }
        assert_eq!(circuit.pulses, (4250, 2750));
    }
}
