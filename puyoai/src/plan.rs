use std::vec::Vec;

use puyoai_core::{
    control::PuyoController, decision::Decision, field::CoreField, kumipuyo::Kumipuyo,
    rensa_result::RensaResult,
};

use crate::es_field::EsCoreField;

#[derive(Clone)]
pub struct Plan {
    field: CoreField,
    decisions: Vec<Decision>,
    rensa_result: RensaResult,
    num_chigiri: usize,
    frames_to_ignite: usize,
    last_drop_frames: usize,
    fallen_ojama: usize,
    fixed_ojama: usize,
    pending_ojama: usize,
    ojama_committing_frame_id: usize,
    has_zenkeshi: bool,
}

impl Plan {
    pub fn new(
        field: CoreField,
        decisions: Vec<Decision>,
        rensa_result: RensaResult,
        num_chigiri: usize,
        frames_to_ignite: usize,
        last_drop_frames: usize,
        fallen_ojama: usize,
        fixed_ojama: usize,
        pending_ojama: usize,
        ojama_committing_frame_id: usize,
        has_zenkeshi: bool,
    ) -> Self {
        Plan {
            field,
            decisions,
            rensa_result,
            num_chigiri,
            frames_to_ignite,
            last_drop_frames,
            fallen_ojama,
            fixed_ojama,
            pending_ojama,
            ojama_committing_frame_id,
            has_zenkeshi,
        }
    }

    pub fn field(&self) -> &CoreField {
        &self.field
    }
    pub fn first_decision(&self) -> &Decision {
        &self.decisions[0]
    }
    pub fn decision(&self, nth: usize) -> &Decision {
        &self.decisions[nth]
    }
    pub fn decisions(&self) -> &Vec<Decision> {
        &self.decisions
    }
    pub fn decision_size(&self) -> usize {
        self.decisions.len()
    }
    pub fn rensa_result(&self) -> &RensaResult {
        &self.rensa_result
    }
    pub fn frames_to_ignite(&self) -> usize {
        self.frames_to_ignite
    }
    pub fn last_drop_frames(&self) -> usize {
        self.last_drop_frames
    }
    pub fn score(&self) -> usize {
        self.rensa_result.score
    }
    pub fn chain(&self) -> usize {
        self.rensa_result.chain
    }
    /// 連鎖が終わるまでのフレーム数
    pub fn frame(&self) -> usize {
        self.rensa_result.frame
    }
    pub fn quick(&self) -> bool {
        self.rensa_result.quick
    }
    pub fn num_chigiri(&self) -> usize {
        self.num_chigiri
    }
    /// 次に操作が可能になるまでのフレーム数（連鎖を発火してたらその後まで）
    pub fn total_frames(&self) -> usize {
        self.frames_to_ignite + self.last_drop_frames + self.rensa_result.frame
    }
    pub fn is_rensa_plan(&self) -> bool {
        self.rensa_result.chain > 0
    }
    pub fn fallen_ojama(&self) -> usize {
        self.fallen_ojama
    }
    pub fn pending_ojama(&self) -> usize {
        self.pending_ojama
    }
    pub fn fixed_ojama(&self) -> usize {
        self.fixed_ojama
    }
    pub fn total_ojama(&self) -> usize {
        self.pending_ojama + self.fixed_ojama
    }
    pub fn ojama_committing_frame_id(&self) -> usize {
        self.ojama_committing_frame_id
    }
    pub fn has_zenkeshi(&self) -> bool {
        self.has_zenkeshi
    }
}

impl Plan {
    pub fn iterate_available_plans_internal<Callback>(
        field: &CoreField,
        seq: &Vec<Kumipuyo>,
        decisions: &mut Vec<Decision>,
        current_depth: usize,
        max_depth: usize,
        current_num_chigiri: usize,
        total_frames: usize,
        callback: &mut Callback,
    ) where
        Callback: FnMut(&CoreField, &Vec<Decision>, usize, usize, usize, bool),
    {
        let kumipuyo = &seq[current_depth];
        let controller = PuyoController::new();

        for decision in {
            if kumipuyo.is_rep() {
                Decision::all_valid_decisions_for_rep()
            } else {
                Decision::all_valid_decisions()
            }
        } {
            if !controller.is_reachable(field, &decision) {
                continue;
            }

            let is_chigiri = field.is_chigiri_decision(decision);
            let drop_frames = field.es_frames_to_drop_next(decision);

            decisions.push(decision.clone());

            let mut next_field = field.clone();
            if !next_field.drop_kumipuyo(decision, kumipuyo) {
                decisions.pop();
                continue;
            }

            let should_fire = next_field.rensa_will_occur_when_last_decision_is(decision);
            if !should_fire && next_field.is_dead() {
                decisions.pop();
                continue;
            }

            if current_depth + 1 == max_depth || should_fire {
                callback(
                    &next_field,
                    decisions,
                    current_num_chigiri + (is_chigiri as usize),
                    total_frames,
                    drop_frames,
                    should_fire,
                )
            } else {
                Plan::iterate_available_plans_internal(
                    &next_field,
                    seq,
                    decisions,
                    current_depth + 1,
                    max_depth,
                    current_num_chigiri + (is_chigiri as usize),
                    total_frames + drop_frames,
                    callback,
                );
            }

            decisions.pop();
        }
    }

    pub fn iterate_available_plans<Callback>(
        field: &CoreField,
        seq: &Vec<Kumipuyo>,
        max_depth: usize,
        callback: &mut Callback,
    ) where
        Callback: FnMut(&Plan),
    {
        // 元の実装では `seq.len() < max_depth` の場合に全配色を試しているが、ここでは省いている
        assert!(seq.len() >= max_depth);

        let mut decisions: Vec<Decision> = vec![];
        decisions.reserve(max_depth);

        let mut f = |field_before_rensa: &CoreField,
                     decisions: &Vec<Decision>,
                     num_chigiri: usize,
                     frames_to_ignite: usize,
                     last_drop_frames: usize,
                     should_fire: bool| {
            debug_assert!(!decisions.is_empty());

            if should_fire {
                let mut cf = field_before_rensa.clone();
                let rensa_result = cf.es_simulate();
                debug_assert!(rensa_result.chain > 0);
                if !cf.is_dead() {
                    callback(&Plan::new(
                        cf,
                        decisions.clone(),
                        rensa_result,
                        num_chigiri,
                        frames_to_ignite,
                        last_drop_frames,
                        0,
                        0,
                        0,
                        0,
                        false,
                    ));
                }
            } else {
                debug_assert!(!field_before_rensa.is_dead());
                let rensa_result = RensaResult::empty();
                callback(&Plan::new(
                    field_before_rensa.clone(),
                    decisions.clone(),
                    rensa_result,
                    num_chigiri,
                    frames_to_ignite,
                    last_drop_frames,
                    0,
                    0,
                    0,
                    0,
                    false,
                ));
            }
        };

        Plan::iterate_available_plans_internal(
            field,
            seq,
            &mut decisions,
            0,
            max_depth,
            0,
            0,
            &mut f,
        );
    }
}

#[cfg(test)]
mod tests {
    use puyoai_core::{
        color::PuyoColor,
        control::PuyoController,
        field,
        field::{BitField, CoreField},
    };

    use super::*;
    use crate::es_frame;

    #[test]
    fn test_iterate_available_plans() {
        let field = CoreField::from_str("  YY  ");
        let seq = vec![
            Kumipuyo::new(PuyoColor::RED, PuyoColor::RED),
            Kumipuyo::new(PuyoColor::YELLOW, PuyoColor::YELLOW),
            Kumipuyo::new(PuyoColor::RED, PuyoColor::RED),
        ];

        let mut found = false;
        Plan::iterate_available_plans(&field, &seq, 3, &mut |plan: &Plan| {
            if !plan.is_rensa_plan() {
                return;
            }
            if plan.rensa_result().chain < 2 {
                return;
            }
            if !plan.field().is_zenkeshi() {
                return;
            }
            found = true;
        });

        assert!(found);
    }

    #[test]
    fn test_iterate_available_plans_issue_8() {
        let heights = [0, 12, 2, 10, 0, 0];
        let field = {
            // TODO: make this part into a separate function?
            let mut bf = BitField::new();
            for y in 0..field::WIDTH {
                for x in 0..heights[y] {
                    bf.set_color(y + 1, x + 1, PuyoColor::OJAMA);
                }
            }
            CoreField::from_bit_field(&bf)
        };

        let seq = vec![Kumipuyo::new(PuyoColor::RED, PuyoColor::RED)];

        let pc = PuyoController::new();
        let mut found = true;
        Plan::iterate_available_plans(&field, &seq, 1, &mut |plan: &Plan| {
            if !pc.is_reachable(&field, plan.first_decision()) {
                dbg!(plan.first_decision());
                found = false;
            }
        });

        assert!(found);
    }

    #[test]
    fn test_iterate_available_plans_with_rensa() {
        let field = CoreField::from_str("  RR  ");
        let seq = vec![
            Kumipuyo::new(PuyoColor::RED, PuyoColor::RED),
            Kumipuyo::new(PuyoColor::BLUE, PuyoColor::BLUE),
        ];

        let mut plan_opt: Option<Plan> = None;
        Plan::iterate_available_plans(&field, &seq, 2, &mut |p: &Plan| {
            if p.decision_size() == 1 && p.first_decision() == &Decision::new(3, 2) {
                plan_opt = Some(p.clone());
            }
        });
        assert!(plan_opt.is_some());

        let plan = plan_opt.unwrap();
        assert_eq!(plan.chain(), 1);
        assert_eq!(plan.score(), 40);
        assert_eq!(plan.frame(), es_frame::FRAMES_CHAIN[0]);
        assert!(plan.quick());

        assert_eq!(
            plan.total_frames(),
            es_frame::FRAMES_GROUNDING[2] + es_frame::FRAMES_CHAIN[0]
        );
        assert_eq!(plan.frames_to_ignite(), 0);
    }

    #[test]
    fn test_iterate_available_plans_num_chigiri() {
        let field = CoreField::from_str("  O   ");
        let seq = vec![
            Kumipuyo::new(PuyoColor::RED, PuyoColor::RED),
            Kumipuyo::new(PuyoColor::RED, PuyoColor::RED),
        ];

        let mut found = false;

        Plan::iterate_available_plans(&field, &seq, 2, &mut |plan: &Plan| {
            if plan.decision(0) == &Decision::new(3, 1) && plan.decision(1) == &Decision::new(3, 1)
            {
                assert_eq!(plan.num_chigiri(), 2);
                found = true;
            }
        });

        assert!(found);
    }
}
