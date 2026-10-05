# Rubensohl (`rubensohl.bid`)

Two switches: `notrump.rubensohl.over_interference` (after they
overcall our 1NT) and `competitive.rubensohl_after_double.play` (after
partner's takeout double of their weak two). Both are on in the
Precision and Precision-14-16 cards, with Lebensohl off. Cases:
`rubensohl.test`.

## What it plays (2026-10-05)

Over a natural 2♦/2♥/2♠ overcall of our 1NT: two of a suit natural, the
double as the card says; 2NT clubs, 3♣ diamonds, 3♦ hearts, 3♥ spades,
3♠ notrump. Transfers to a minor: six or more (weak to play, or game
values then 3NT). To a major: five or more, invitational or better. The
transfer to their suit is Stayman (game values, four of an unbid
major); the transfer to notrump is game values without a stopper and
without a four-card major; 3NT game with a stopper. Opener completes
with a minimum (15-16); with 17 bids game with three of the major, 3NT
without. Responder after the completion: pass weak or invitational,
game with six of the major, 3NT with five (opener corrects with three).

After our double of their weak two the same transfers from 2NT, the
doubler in opener's place: four-card minors (weak to play at three, or
9+ with five), majors 9-12 (over 2♠ the heart transfer is any hand up to
12), the transfer to their suit Stayman with 13+, 3♠ game without a
stopper, 3NT with one.

Switching it on turns off Lebensohl over 1NT (notrump/lebensohl.bid,
`!rub` on its contexts), the natural 2NT invitation over their
overcall (notrump/nt-interference.bid, `!rub`), and Lebensohl and the
natural three-level replies after the double (vs-preempts.bid,
`!(leb | rub)`). The natural three-level forcing bids of
nt-interference.bid stay, outranked by the transfers (priority 3).

## Deviations and open questions

- Over their 2♣ (natural) nothing changes: like our Lebensohl, Rubensohl
  reads only 2♦/2♥/2♠.
- "Rubensohl after 1m" (BBA key, on in the Precision cards) has no card
  field; not written.
- Sources disagree in detail: Wikipedia's Rubinsohl (2NT clubs and up,
  3♣ over 2♦ is Stayman) is what we play; Larry Cohen's and Todd's
  "Transfer Lebensohl" keep a 2NT relay. Question for Rick if the
  Precision partnership plays the other.

## Compare (2026-10-05)

Lebensohl and Lebensohl2 with `--set notrump.rubensohl.over_interference=true`
(84.8% agreement), Lebensohl_vs_Opps_W2_* with
`--set competitive.rubensohl_after_double.play=true` (85.2%): the
divergences at our calls are the transfers against BBA's Lebensohl
(1NT 2♠: our 3♦ transfer where BBA bids a natural 3♥, our 3♥ Stayman
where BBA makes a negative double). No forced passes or contradictions.
Tripwire: three boards in Weak_NT_11-16, SCS_Two_Clubs and
Weak_NT_14-15 (Precision cards); in the last, 1NT 2♥ 2NT (natural
invitation) became a pass, since Rubensohl has no natural 2NT.
Question for Rick: invite in notrump by double, or keep it as is.

## Sources

- Wikipedia, "Rubinsohl" (https://en.wikipedia.org/wiki/Rubinsohl):
  the transfer scheme over 2♦, "at least invitational" major transfers,
  3♠ game without a stopper, the same scheme after the double of a weak
  two.
- Larry Cohen, "Transfer Lebensohl"
  (https://www.larryco.com/bridge-articles/transfer-lebensohl), and
  Robert S. Todd, Advancing in Bridge 548: opener completes with a
  minimum, any other bid is forcing.
