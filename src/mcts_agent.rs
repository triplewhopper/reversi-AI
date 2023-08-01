use crate::agent::Agent;
use crate::board::{Action, Board, Player, Pos};
use rand::seq::IteratorRandom;
// use mcts;

pub struct MCTSAgent {
    tree: RefCell<MCTS>,
    cursor: Rc<RefCell<MCTSNode>>,
    n_simulations: u32,
    exploration: f32,
}
impl MCTSAgent {
    pub fn new(n_simulations: u32, c: f32) -> Self {
        let tree = MCTS::new(Board::initial(), Player::Black);
        let cursor = tree.root.clone();
        Self {
            n_simulations,
            exploration: c,
            cursor,
            tree: RefCell::new(tree),
        }
    }
}
impl Agent for MCTSAgent {
    fn initialize(&mut self) {
        self.cursor = self.tree.borrow().root.clone();
    }
    fn opponent_move_callback(&mut self, action: Option<Action>) {
        let new_cursor = self.tree.borrow().force_expand_on_action(&self.cursor, action);
        new_cursor.map_or_else(|| panic!("new_cursor is none"), |x| self.cursor = x);
        eprintln!("root.n_visits = {}, cursor.n_visits={}", self.tree.borrow().root.borrow().n_visits.get(), self.cursor.borrow().n_visits.get());
        eprintln!("N_NODES={}", unsafe { N_NODES });
    }
    // Monte Carlo Tree Search
    fn select_move(&mut self, board: &Board, p: Player) -> Option<Action> {
        // {
        //     eprintln!("{}", m.root.borrow());
        // }
        assert_eq!(self.cursor.borrow().state.player, p);
        assert_eq!(self.cursor.borrow().state.board, *board);
        // eprintln!("self.cursor = {}", self.cursor.borrow());
        let best_node = self.tree.borrow_mut().best_action(&self.cursor, self.n_simulations, self.exploration);
        best_node.map(|x| {
            // eprintln!("best_node = {}", x.borrow());
            assert_ne!(self.cursor.borrow().state.player, x.borrow().state.player);
            let action = x.borrow().causing_action;
            self.cursor = x;
            action
        }).flatten()
    }
}

#[derive(Clone, Debug)]
struct OthelloState {
    board: Board,
    n_steps: u32,
    player: Player,
}

impl OthelloState{
    fn valid_moves(&self) -> impl Iterator<Item = Action> + '_ {
        self.board.valid_moves(self.player)
    }
}
use std::collections::VecDeque;
use std::fmt::Display;
use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell};

#[derive(Debug)]
struct MCTSNode {
    state: OthelloState,
    parent: Weak<RefCell<MCTSNode>>,
    children: Vec<Rc<RefCell<MCTSNode>>>,
    n_visits: Cell<u32>,
    reward: Cell<f32>,
    causing_action: Option<Action>,
    untried_actions: Option<Vec<Action>>, // None if pass
}
impl std::fmt::Display for MCTSNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "MCTSNode (q={}, n={}) {:?}'s turn, board = {:?}", self.reward.get(), self.n_visits.get(), self.state.player, self.state.board)?;
        writeln!(f, "tried = {:?}", self.children.iter().map(|x| x.borrow().causing_action.unwrap().at).collect::<Vec<Pos>>())?;
        writeln!(f, "untried = {:?}", self.untried_actions.as_ref().map(|a| a.iter().map(|x| x.at).collect::<Vec<Pos>>()))?;
        writeln!(f)?;
        Ok(())
    }
}
static mut N_NODES: u64 = 0;

impl MCTSNode {
    fn new_root(board: Board, player: Player) -> Self {
        let res = Self {
            state: OthelloState {
                board,
                n_steps: 0,
                player,
            },
            parent: Weak::new(),
            children: Vec::new(),
            n_visits: Cell::new(0),
            reward: Cell::new(0.),
            causing_action: None,
            untried_actions: {
                let moves: Vec<_> = board.valid_moves(player).collect();
                // moves.reverse();
                if moves.is_empty() {
                    None
                } else {
                    Some(moves)
                }
            },
        };
        res
    }

    fn new(state: OthelloState, parent: &Rc<RefCell<MCTSNode>>, causing_action: Option<Action>) -> Self {
        unsafe {
            N_NODES += 1;
        }
        assert_ne!(state.player, parent.borrow().state.player);
        assert_eq!(
            parent.borrow().untried_actions.is_none(), causing_action.is_none(), "parent=\n{:#?}, causing_action={:?}", parent, causing_action);
        let untried_actions: Vec<_> = state.valid_moves().collect();
        let res = Self {
            state,
            parent: Rc::downgrade(parent),
            children: Vec::new(),
            n_visits: std::cell::Cell::new(0),
            reward: std::cell::Cell::new(0.),
            causing_action, // None for pass or root
            untried_actions: {
                if untried_actions.is_empty() {
                    None
                } else {
                    Some(untried_actions)
                }
            }
        };
        res
    }
    /// Q(v) – **Total simulation reward** is an attribute of a node *v* and 
    /// in a simplest form is a sum of simulation results that passed through considered node.
    /// c.f. https://int8.io/monte-carlo-tree-search-beginners-guide/
    fn q(&self) -> f32 { 
        self.reward.get()
    }
    /// N(v) – **Total number of visits** is another attribute of a node *v* 
    /// representing a counter of how many times a node has been on the 
    /// backpropagation path (and so how many times it contributed to the 
    /// total simulation reward)
    fn n(&self) -> u32 {
        self.n_visits.get()
    }

    fn is_terminal_node(&self) -> bool {
        self.state.board.is_final()
    }

    fn rollout(&self) -> i32 {
        let mut current_rollout_state = self.state.clone();
        let mut passed = false;
        loop {
            if let Some(action) = Self::rollout_policy(&current_rollout_state){
                passed = false;
                current_rollout_state = OthelloState {
                    board: action.board,
                    n_steps: current_rollout_state.n_steps + 1,
                    player: current_rollout_state.player.opponent(),
                }
            } else {
                if passed {
                    break;
                }
                passed = true;
                current_rollout_state = OthelloState {
                    board: current_rollout_state.board,
                    n_steps: current_rollout_state.n_steps + 1,
                    player: current_rollout_state.player.opponent(),
                }
            }
            
        }
        let board = current_rollout_state.board;
        let diff = board.count(self.state.player) as i32 - board.count(self.state.player.opponent()) as i32;
        return diff.signum()
    }

    fn rollout_policy(state: &OthelloState) -> Option<Action> {
        let valid_moves_mask = state.board.valid_moves_fast(state.player);
        for p in [Pos::A1, Pos::A8, Pos::H1, Pos::H8] {
            if valid_moves_mask & (1u64 << p as u64) != 0 {
                return Some(Action {
                    at: p,
                    board: state.board.place_at_unchecked(state.player, p)
                });
            }
        }
        state.valid_moves().choose(&mut rand::thread_rng())
    }

    fn has_been_expanded_on(&self, action: Option<Action>) -> Option<Rc<RefCell<MCTSNode>>> {
        self.children.iter().find_map(|x| if x.borrow().causing_action == action { Some(x.clone()) } else { None })
    }

    // fn is_fully_expanded(&self) -> bool {
        // todo: double pass is not considered
    //     self.untried_actions.as_ref().map_or_else(|| !self.children.is_empty(), |actions| actions.is_empty())
    // }

 

}

#[derive(Debug)]
struct MCTS {
    root: Rc<RefCell<MCTSNode>>,
    // nodes: Vec<RefCell<MCTSNode>>,
    // n_simulations: u32,
}

impl MCTS {
    fn new(board: Board, p: Player) -> Self {
        let root = MCTSNode::new_root(board, p);
        Self {
            root: Rc::new(RefCell::new(root)),
            // nodes: Vec::new(),
            // n_simulations,
        }
    }
    // fn move_downward(&mut self, action: Action) -> Rc<RefCell<MCTSNode>> {
    //     let mut node = self.root;
    //     let mut state = node.borrow().state.clone();
    //     state.board = action.board;
    //     state.n_steps += 1;
    //     state.player = state.player.opponent();
    //     let child = Rc::new(RefCell::new(MCTSNode::new(state, &node, action)));
    //     node.borrow_mut().children.push(Rc::clone(&child));
    //     child
    // }
    fn tree_policy(& mut self, start: &Rc<RefCell<MCTSNode>>, c: f32) -> Option<Rc<RefCell<MCTSNode>>> {
        let mut current_node = start.clone();
        loop {
            if current_node.borrow().is_terminal_node() {
                return Some(current_node);
            }
            // let mut m: std::cell::RefMut<'a, _> = current_node.borrow_mut();
            if let Some(node) = self.expand(&current_node) {
                // current_node.borrow_mut().children.push(node.clone());
                return Some(node);
            } else {
                match self.best_uct_child(&current_node, c) {
                    None => panic!("No best child: current_node=\n{}", current_node.borrow()),
                    Some(child) => {
                        
                        current_node = child;
                    },
                };
            }
        }
    }

    fn best_action(&mut self, start: &Rc<RefCell<MCTSNode>>, n_simulations: u32, c: f32) -> Option<Rc<RefCell<MCTSNode>>>{
        // eprintln!("simulating (q={}, n={}) from board = {:?}", self.root.borrow().q(), self.root.borrow().n(), self.root.borrow().state.board);
        for _i in 0..n_simulations {
            if let Some(v) = self.tree_policy(start, c) {
                let reward = v.borrow().rollout();
                self.backpropagate(&v, reward);
            } /*else {
                break;
            }*/
        }
        let best_child = self.best_child(start);
        
        // {
            
        //     eprintln!("simulated (q={}, n={})", self.root.borrow().q(), self.root.borrow().n());
        //     self.root.borrow().children.iter().for_each(|x| {
        //         eprintln!("tried {} (q={}, n={}) uct={}", 
        //         x.borrow().causing_action.unwrap().at, x.borrow().q(), x.borrow().n(), uct(&x.borrow(), &self.root.borrow(), c));
        //     });
        //     best_child.as_ref().map(
        //         |x| eprintln!("best child = {:?}, uct = {}", x.borrow().causing_action.unwrap().at, uct(&x.borrow(), &self.root.borrow(), c))
        //     );
        //     let mut buf = String::new();
        //     std::io::stdin().read_line(&mut buf).unwrap();
        // }
        return best_child;
    }

    /// return the best child of *node* with respect to the *Q/N* value.
    /// 
    /// return `None` if *node* has no children.
    fn best_child(&self, node: &Rc<RefCell<MCTSNode>>) -> Option<Rc<RefCell<MCTSNode>>> {
        node.borrow().children.iter().max_by(|x, y| {
            let x = x.borrow();
            let y = y.borrow();
            let score_x = -x.q() / x.n() as f32; // don't know why, but it works
            let score_y = -y.q() / y.n() as f32;
            score_x.total_cmp(&score_y)
        }).map(|x| Rc::clone(x))
    }

    /// select one of the children of *self* giving the highest *UCT* value, given a constant *c* as the exploration parameter.
    fn best_uct_child(& self, node: &Rc<RefCell<MCTSNode>>, c: f32) -> Option<Rc<RefCell<MCTSNode>>> {
        let acc = (f32::NEG_INFINITY, None);
        node.borrow().children.iter().fold (acc, |acc, x| {
            let u = uct(&(*x).borrow(), &(*node).borrow(), c);
            if u > acc.0 {
                (u, Some(x)) 
            } else {
                acc
            }
        }).1.map(|x| Rc::clone(x))
        // acc.1
    }

    /// this method creates a child node transiting from current `node` and does not check if `action` is valid.
    /// 
    /// `action` is `None` if and only if no valid action is available, i.e. the player passes.
    fn expand_impl(&self, node: &Rc<RefCell<MCTSNode>>, action: Option<Action>) -> Rc<RefCell<MCTSNode>> {
        assert_eq!(action.is_none(), node.borrow().untried_actions.is_none());
        let new_state = OthelloState {
            board: action.map(|x| x.board).unwrap_or(node.borrow().state.board),
            player: node.borrow().state.player.opponent(),
            n_steps: node.borrow().state.n_steps + 1,
        };
        let child_node = Rc::new(RefCell::new(MCTSNode::new(new_state, node, action)));
        node.borrow_mut().children.push(child_node.clone());
        child_node
    }
    fn expand(&self, node: &Rc<RefCell<MCTSNode>>) -> Option<Rc<RefCell<MCTSNode>>>{
        let mut nbm = node.borrow_mut();
        match nbm.untried_actions {
            Some(ref mut untried_actions) => {
                if let Some(action) = untried_actions.pop() {
                    std::mem::drop(nbm);
                    return Some(self.expand_impl(node, Some(action)));
                }
                return None;
            }
            None => {
                std::mem::drop(nbm);
                if !node.borrow().children.is_empty() {
                    return None; // fully expanded
                }

                // PASS
                // Double pass is impossible since is_terminal_node() is tested first
                return Some(self.expand_impl(node, None));
            }
        }
    }

    /// Force an expansion on `node` with `action`. if `action` is `None` then it is a pass.
    /// 
    /// returns the child node (may already exist, or newly created)
    /// 
    /// returns `None` if `action` is not valid.
    fn force_expand_on_action(&self, node: &Rc<RefCell<MCTSNode>>, action: Option<Action>) -> Option<Rc<RefCell<MCTSNode>>> {
        if let Some(child) = node.borrow().has_been_expanded_on(action) {
            return Some(child);
        }
        let mut nbm = node.borrow_mut(); 
        match (&mut nbm.untried_actions, action){
            (None, None)=> {
                assert!(action.is_none()); 
                std::mem::drop(nbm);
                return Some(self.expand_impl(node, None));
            },
            (Some(untried_actions), Some(action))=> {
                if let Some(index) = untried_actions.iter().position(|x| x.at == action.at) {
                    let action= untried_actions.swap_remove(index);
                    std::mem::drop(nbm);
                    return Some(self.expand_impl(node, Some(action)));
                }
                std::mem::drop(nbm);
                panic!("action {:?} is not in untried_actions, node={}", action, node.borrow());
            }
            (_, y) => {
                std::mem::drop(nbm);

                panic!("untried_actions is {:?} but action is {:?}, node is {}", node.borrow().untried_actions, y, node.borrow());
            }, 
        }
    }

    fn backpropagate(&self, node: &Rc<RefCell<MCTSNode>>, mut result: i32) {
        let mut node = node.clone();
        loop {
            let x = node.borrow();
            x.n_visits.replace(x.n_visits.get() + 1);
            x.reward.replace(x.reward.get() + result as f32);
            result = - result;
            match x.parent.upgrade() {
                None => break,
                Some(parent) => {
                    assert!(parent.borrow().state.player != node.borrow().state.player);
                    std::mem::drop(x); 
                    node = parent;
                },
            }
        }
    }


}

/// Upper Confidence Bound of a transition from *v* to *v<sub>i</sub>*, given a constant c.
fn uct(vi: &MCTSNode, v: &MCTSNode, c: f32) -> f32 {
    assert_ne!(vi.state.player, v.state.player);
    vi.q() as f32 / vi.n() as f32 + c * ((v.n() as f32).ln() / (vi.n() as f32)).sqrt()
}
