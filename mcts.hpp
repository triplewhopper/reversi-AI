#pragma once

#include <memory>
#include "board.hpp"

template <Player P>
struct GameState {
    Board board;
    size_t n_steps;
    static constexpr Player player = P;
};

enum class Reward {
    Win = 1,
    Loss = -1,
    Draw = 0
};


template <Player P>
struct MCTSNode {
    GameState<P> state;
    const MCTSNode<!P> *parent;
    std::vector<MCTSNode<!P>> children;
    size_t n_visits;
    double reward;
    uint64_t untried_actions;
    std::unique_ptr<Action> causing_action;
    
    MCTSNode(GameState<P> const& state, MCTSNode<!P> &parent, std::unique_ptr<Action> causing_action)
        : state(state), parent(&parent), children{}, n_visits(0), reward(0.), 
        causing_action(std::move(causing_action)), untried_actions(state.board.valid_moves(P)) {
            assert((parent.untried_actions == 0) == (causing_action.get() == nullptr));
            assert(parent.state.n_steps + 1 == state.n_steps);
        }

    MCTSNode(GameState<P> const& state): state(state), parent(nullptr), children(), n_visits(0), reward(0.0), untried_actions(state.board.valid_moves(P)) {}
    double q() const { return reward; }
    size_t n() const { return n_visits; }
    bool is_terminal_node() const { return state.board.valid_moves(P) == 0; }
    Reward rollout() const;
    static std::unique_ptr<Action> rollout_policy(GameState<P> const& state);
    MCTSNode<!P> *has_been_expanded_on(std::unique_ptr<Action> action) const;
};

template <Player P>
class MCTS {
    private:
    MCTSNode<P> *expand_impl(MCTSNode<P> &node, std::unique_ptr<Action> action) &;
    public:
    MCTSNode<P> root;
    MCTS(GameState<P> const& state): root(state) {}
    MCTSNode<P> *best_action(MCTSNode<P> &start, size_t n_simulations, double c) &;
    MCTSNode<P> *best_child(MCTSNode<P> &node) &;
    MCTSNode<P> *best_uct_child(MCTSNode<P> &node, double c) &;
    MCTSNode<P> *expand(MCTSNode<P> &node) &;
    MCTSNode<P> *tree_policy(MCTSNode<P> &start, double c = 1.4) &;
    MCTSNode<P> *force_expand_on_action(MCTSNode<P> &node, std::unique_ptr<Action> action) &;
    void backpropagate(MCTSNode<P> &node, double reward) &;
};