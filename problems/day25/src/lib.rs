use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::collections::{HashMap, HashSet, VecDeque};
use tokio::sync::mpsc::Receiver;

const NODESAMPLE: usize = 35;
const BRIDGES: usize = 3;

type Graph = HashMap<String, Vec<String>>;

async fn parse_line(line: &str) -> (String, Vec<String>) {
    let (node, edges) = line.split_once(": ").unwrap();
    (
        node.to_string(),
        edges.split(' ').map(|s| s.to_string()).collect(),
    )
}

fn both_ways(graph: &Graph) -> Graph {
    let mut undirected_graph = graph.clone();

    for (node, edges) in graph.iter() {
        for edge in edges.iter() {
            let new_node = undirected_graph.entry(edge.clone()).or_default();
            new_node.push(node.clone());
        }
    }

    undirected_graph
}

fn find_shortest_path(graph: &Graph, start: &str, end: &str) -> Option<Vec<String>> {
    if start == end {
        return Some(vec![start.to_string()]);
    }

    let mut queue = VecDeque::new();
    queue.push_back(vec![start.to_string()]);

    while let Some(history) = queue.pop_front() {
        let current = history.last().unwrap();
        if let Some(neighbors) = graph.get(current) {
            for neighbor in neighbors {
                if !history.contains(neighbor) {
                    let mut new_hist = history.clone();
                    new_hist.push(neighbor.clone());

                    if neighbor == end {
                        return Some(new_hist);
                    }
                    queue.push_back(new_hist);
                }
            }
        }
    }

    None
}

fn find_bridges(graph: &Graph, n_bridges: usize) -> Vec<(String, String)> {
    let seed = [42; 32];
    let mut rng = StdRng::from_seed(seed);

    let mut nodes: Vec<_> = graph.keys().cloned().collect();
    nodes = nodes
        .choose_multiple(&mut rng, NODESAMPLE)
        .cloned()
        .collect();

    let mut shortest_paths = Vec::new();

    for (idx1, node1) in nodes.iter().enumerate() {
        for (idx2, node2) in nodes.iter().enumerate() {
            if idx1 >= idx2 {
                continue;
            }
            if let Some(path) = find_shortest_path(graph, node1, node2) {
                shortest_paths.push(path);
            }
        }
    }

    let mut edges = HashMap::new();
    for path in shortest_paths.into_iter() {
        for edge in path.windows(2) {
            let mut edge = edge.to_vec();
            edge.sort_unstable();
            let counter = edges.entry(edge).or_insert(0);
            *counter += 1;
        }
    }

    let mut edges: Vec<_> = edges.into_iter().collect();
    edges.sort_by(|(_edge, count), (_edge2, count2)| count.cmp(count2));

    edges
        .into_iter()
        .rev()
        .take(n_bridges)
        .map(|(edge, _count)| (edge[0].clone(), edge[1].clone()))
        .collect()
}

fn remove_edges(graph: &Graph, to_remove: &[(String, String)]) -> Graph {
    let mut graph = graph.clone();

    for (node, edges) in graph.iter_mut() {
        for (node1, node2) in to_remove.iter() {
            if node == node1 {
                edges.retain(|e| e != node2);
            } else if node == node2 {
                edges.retain(|e| e != node1);
            }
        }
    }

    graph
}

fn find_networks(graph: &Graph) -> Vec<Vec<String>> {
    let mut networks = Vec::new();
    let mut visited = HashSet::new();

    for node in graph.keys() {
        if !visited.contains(node) {
            let mut network = Vec::new();
            let mut stack = VecDeque::new();
            stack.push_back(node);

            while let Some(current) = stack.pop_back() {
                if visited.insert(current.clone()) {
                    network.push(current.clone());
                    if let Some(neighbors) = graph.get(current) {
                        for neighbor in neighbors {
                            if !visited.contains(neighbor) {
                                stack.push_back(neighbor);
                            }
                        }
                    }
                }
            }

            networks.push(network);
        }
    }

    networks
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();
    while let Some(line) = rx.recv().await {
        if line.is_empty() {
            continue;
        }
        let task = tokio::spawn(async move { parse_line(&line).await });
        tasks.push(task);
    }

    let mut graph = HashMap::new();
    for task in tasks {
        if let Ok(node) = task.await {
            graph.insert(node.0, node.1);
        }
    }

    graph = both_ways(&graph);
    let remove_bridges = find_bridges(&graph, BRIDGES);
    graph = remove_edges(&graph, &remove_bridges);

    let part1 = find_networks(&graph)
        .into_iter()
        .map(|n| n.len())
        .collect::<Vec<usize>>();

    dbg!(part1);
}
