# Kinship 表現候補

対応目安:

- `core`: 現在または近い実装で API/DB の基本 kind として扱う価値が高い。
- `derived`: 保存済み edge、性別、生年月日、path から read-time に導出する。
- `extended`: 表現可能だが、subtype / role / legal context などの拡張 model が必要。
- `display`: domain kind としては増やさず、表示ラベルで表現する。
- `out-of-scope`: 家系 graph の主 contract ではなく、別 resource / metadata として扱う方がよい。

原則:

- DB に保存するのは事実関係の最小集合に寄せる。
- `father` / `mother` / `son` / `daughter` のような性別派生は `parent` / `child` + node gender で表現する。
- `older_*` / `younger_*` は生年月日または年齢順から導出する。
- `relation_to_center` は粗い machine-readable token とし、詳細な続柄は `relation_path_to_center`、edge kind、node attributes から導出する。

1. まず保存対象にしたい直接関係

| key候補 | 英名 | 和名 | 備考 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- | --- |
| `parent` | parent | 親 | 汎用 | `core` | `edges[].kind = "parent"`。center から見て親なら `relation_to_center = "parent"` / offset `-1`。 |
| `child` | child | 子 | `parent` の逆向き | `derived` | 保存は `parent(parent -> child)` に統一し、逆向き path で `child` として表示。 |
| `parent_child` | parent-child relationship | 親子関係 | API/DBではこれを推奨 | `display` | API/DB kind ではなく、`parent` edge の概念名として扱う。 |
| `biological_parent` | biological parent | 実親 / 生物学上の親 | 血縁・遺伝上の親 | `extended` | `parent` の subtype / role metadata として表現。core kind に増やすかは別 RFC で決める。 |
| `biological_child` | biological child | 実子 / 生物学上の子 | 血縁・遺伝上の子 | `derived` | 対応する `biological_parent` の逆向きとして導出。 |
| `birth_parent` | birth parent | 生みの親 | 出産・出生上の親 | `extended` | `parent` の subtype / role metadata として表現。core kind に増やすかは別 RFC で決める。 |
| `birth_child` | birth child | 生まれた子 | 出生上の子 | `derived` | 対応する `birth_parent` の逆向きとして導出。 |
| `genetic_parent` | genetic parent | 遺伝上の親 | 生殖補助医療を厳密に扱う場合 | `extended` | `parent` の subtype / role metadata として表現。core kind に増やすかは別 RFC で決める。 |
| `genetic_child` | genetic child | 遺伝上の子 | 同上 | `derived` | 対応する `genetic_parent` の逆向きとして導出。 |
| `legal_parent` | legal parent | 法的な親 | 戸籍・法制度上の親 | `extended` | `parent` の subtype / role metadata として表現。core kind に増やすかは別 RFC で決める。 |
| `legal_child` | legal child | 法的な子 | 戸籍・法制度上の子 | `derived` | 対応する `legal_parent` の逆向きとして導出。 |
| `adoptive_parent` | adoptive parent | 養親 | 既存設計にもある | `core` | `edges[].kind = "adoptive_parent"`。lineage edge として扱える。 |
| `adoptive_child` | adoptive child | 養子 / 養女 / 養息子 | 既存設計にもある | `derived` | 対応する `adoptive_parent` の逆向きとして導出。 |
| `step_parent` | step parent | 継親 | 再婚相手などによる親 | `core` | `edges[].kind = "step_parent"`。lineage root 計算に含めるかは traversal policy で決める。 |
| `step_child` | step child | 継子 | 継親の子 | `derived` | 対応する `step_parent` の逆向きとして導出。 |
| `foster_parent` | foster parent | 里親 | 養子縁組とは別 | `extended` | `parent` の subtype / role metadata として表現。core kind に増やすかは別 RFC で決める。 |
| `foster_child` | foster child | 里子 | 同上 | `derived` | 対応する `foster_parent` の逆向きとして導出。 |
| `guardian` | guardian | 後見人 | 家系図というより保護関係 | `extended` | 専用 edge または parent role metadata で表現可能。genealogy core とは分けて扱うのが安全。 |
| `ward` | ward | 被後見人 | guardian の相手 | `derived` | `guardian` edge の逆向きとして導出。 |
| `custodial_parent` | custodial parent | 監護親 | 離婚・別居時に有用 | `extended` | `parent` の subtype / role metadata として表現。core kind に増やすかは別 RFC で決める。 |
| `non_custodial_parent` | non-custodial parent | 非監護親 | 同上 | `extended` | `parent` の subtype / role metadata として表現。core kind に増やすかは別 RFC で決める。 |
| `social_parent` | social parent | 社会的な親 | 法的・血縁ではないが親役割 | `extended` | `parent` の subtype / role metadata として表現。core kind に増やすかは別 RFC で決める。 |
| `de_facto_parent` | de facto parent | 事実上の親 | 実態ベース | `extended` | `parent` の subtype / role metadata として表現。core kind に増やすかは別 RFC で決める。 |
| `intended_parent` | intended parent | 意図された親 | 代理出産・生殖補助医療向け | `out-of-scope` | 出自・医療・契約情報寄り。graph edge ではなく別 resource / provenance metadata 推奨。 |
| `gestational_parent` | gestational parent | 妊娠・出産した親 | 代理母などを区別する場合 | `extended` | `parent` の subtype / role metadata として表現。core kind に増やすかは別 RFC で決める。 |
| `sperm_donor` | sperm donor | 精子提供者 | 続柄より出自情報寄り | `out-of-scope` | 出自・医療・契約情報寄り。graph edge ではなく別 resource / provenance metadata 推奨。 |
| `egg_donor` | egg donor | 卵子提供者 | 同上 | `out-of-scope` | 出自・医療・契約情報寄り。graph edge ではなく別 resource / provenance metadata 推奨。 |
| `embryo_donor` | embryo donor | 胚提供者 | 同上 | `out-of-scope` | 出自・医療・契約情報寄り。graph edge ではなく別 resource / provenance metadata 推奨。 |
| `surrogate_parent` | surrogate parent | 代理出産者 | 厳密には親子関係とは別扱い推奨 | `out-of-scope` | 出自・医療・契約情報寄り。graph edge ではなく別 resource / provenance metadata 推奨。 |
| `unknown_parent` | unknown parent | 不明な親 | プレースホルダー用 | `extended` | placeholder entity + `source_confidence = "unknown"`。専用 kind は不要。 |
| `possible_parent` | possible parent | 親の可能性がある人物 | 仮説・未確定データ用 | `extended` | `parent` edge + `source = "suggested"` / `source_confidence = "unknown"` または `"conflicting"`。 |

2. 配偶者・パートナー・同居関係

| key候補 | 英名 | 和名 | 備考 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- | --- |
| `spouse` | spouse | 配偶者 | 既存設計にある | `core` | `edges[].kind = "spouse"`。symmetric edge として正規化。 |
| `husband` | husband | 夫 | 表示用の性別派生 | `display` | `spouse` + node gender から表示。 |
| `wife` | wife | 妻 | 表示用の性別派生 | `display` | `spouse` + node gender から表示。 |
| `ex_spouse` | ex-spouse | 元配偶者 | 既存設計にある | `derived` | 基本 edge + `end_date` / `end_reason` / 相手の `death_date` から導出。 |
| `ex_husband` | ex-husband | 元夫 | 表示用 | `display` | ended `spouse` + node gender から表示。 |
| `ex_wife` | ex-wife | 元妻 | 表示用 | `display` | ended `spouse` + node gender から表示。 |
| `divorced_spouse` | divorced spouse | 離婚した配偶者 | `ex_spouse` とほぼ同義 | `derived` | 基本 edge + `end_date` / `end_reason` / 相手の `death_date` から導出。 |
| `separated_spouse` | separated spouse | 別居中の配偶者 | 法的婚姻継続中 | `extended` | `spouse` の status metadata が必要。`end_date` とは別に持つ。 |
| `deceased_spouse` | deceased spouse | 死別した配偶者 | `end_reason = death` でも表現可 | `derived` | 基本 edge + `end_date` / `end_reason` / 相手の `death_date` から導出。 |
| `partner` | partner | パートナー | 既存設計にある | `core` | `edges[].kind = "partner"`。 |
| `domestic_partner` | domestic partner | 事実婚パートナー | 法的婚姻でない場合 | `extended` | `partner` + legal status / jurisdiction metadata。 |
| `civil_partner` | civil partner | シビルパートナー | 国・制度依存 | `extended` | `partner` + legal status / jurisdiction metadata。 |
| `fiance` | fiancé / fiancée | 婚約者 | 婚姻前 | `extended` | 専用 edge 追加で表現可能。core graph では優先度低。 |
| `cohabitant` | cohabitant | 同居人 | 要件では定義対象に含まれている | `core` | `edges[].kind = "cohabitant"`。親族とは限らない。 |
| `former_partner` | former partner | 元パートナー | 婚姻ではない解消済み関係 | `derived` | 基本 edge + `end_date` / `end_reason` / 相手の `death_date` から導出。 |
| `former_cohabitant` | former cohabitant | 元同居人 | 同居解消済み | `derived` | 基本 edge + `end_date` / `end_reason` / 相手の `death_date` から導出。 |

3. 兄弟姉妹系

| key候補 | 英名 | 和名 | 備考 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- | --- |
| `sibling` | sibling | 兄弟姉妹 / きょうだい | 汎用 | `derived` | 1 人以上の parent/adoptive_parent 等を共有する node pair から導出。 |
| `brother` | brother | 兄弟 | 性別派生 | `display` | `sibling` + node gender male。 |
| `sister` | sister | 姉妹 | 性別派生 | `display` | `sibling` + node gender female。 |
| `older_sibling` | older sibling | 兄 / 姉 / 年上のきょうだい | 年齢派生 | `derived` | `sibling` + birth_date ordering。 |
| `younger_sibling` | younger sibling | 弟 / 妹 / 年下のきょうだい | 年齢派生 | `derived` | `sibling` + birth_date ordering。 |
| `older_brother` | older brother | 兄 | 表示用 | `display` | `sibling` + male + birth_date が center より早い。 |
| `younger_brother` | younger brother | 弟 | 表示用 | `display` | `sibling` + male + birth_date が center より遅い。 |
| `older_sister` | older sister | 姉 | 表示用 | `display` | `sibling` + female + birth_date が center より早い。 |
| `younger_sister` | younger sister | 妹 | 表示用 | `display` | `sibling` + female + birth_date が center より遅い。 |
| `full_sibling` | full sibling | 両親を同じくする兄弟姉妹 / 全血きょうだい | 例にあるもの。通常は親集合から計算 | `derived` | 2 人の親集合が同じ場合に導出。 |
| `half_sibling` | half sibling | 半血兄弟姉妹 / 異父母きょうだい | 片方の親だけ同じ | `derived` | 親集合の交差が 1、差分がある場合に導出。 |
| `maternal_half_sibling` | maternal half sibling | 異父兄弟姉妹 | 母が同じ | `derived` | 共有 parent の gender から導出。birth/legal subtype が必要なら extended。 |
| `paternal_half_sibling` | paternal half sibling | 異母兄弟姉妹 | 父が同じ | `derived` | 共有 parent の gender から導出。birth/legal subtype が必要なら extended。 |
| `adoptive_sibling` | adoptive sibling | 養兄弟姉妹 | 養親を通じたきょうだい | `derived` | 共有する adoptive/step/foster parent、または spouse-child path から導出。 |
| `step_sibling` | step sibling | 継兄弟姉妹 / 義理の兄弟姉妹 | 親の再婚相手の子 | `derived` | 共有する adoptive/step/foster parent、または spouse-child path から導出。 |
| `foster_sibling` | foster sibling | 里兄弟姉妹 | 里親家庭でのきょうだい | `derived` | 共有する adoptive/step/foster parent、または spouse-child path から導出。 |
| `legal_sibling` | legal sibling | 法的な兄弟姉妹 | 法的親子関係を共有 | `extended` | `legal_parent` subtype を共有する場合に導出。 |
| `biological_sibling` | biological sibling | 血縁上の兄弟姉妹 | 実親を共有 | `extended` | `biological_parent` subtype を共有する場合に導出。 |
| `sibling_in_law` | sibling-in-law | 義兄弟姉妹 | 配偶者の兄弟姉妹、または兄弟姉妹の配偶者 | `derived` | spouse -> sibling または sibling -> spouse path。性別・年齢は表示派生。 |
| `brother_in_law` | brother-in-law | 義兄 / 義弟 | 姻族 | `display` | spouse -> sibling または sibling -> spouse path。性別・年齢は表示派生。 |
| `sister_in_law` | sister-in-law | 義姉 / 義妹 | 姻族 | `display` | spouse -> sibling または sibling -> spouse path。性別・年齢は表示派生。 |

4. 祖先・子孫系

tree_path が ancestor_id / descendant_id / depth を持つ設計なので、祖先・子孫は基本的に派生できます。


| key候補 | 英名 | 和名 | 備考 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- | --- |
| `ancestor` | ancestor | 祖先 | 汎用 | `derived` | lineage path で center より上。`relation_to_center = "ancestor"`、offset `< 0`。 |
| `descendant` | descendant | 子孫 | 汎用 | `derived` | lineage path で center より下。`relation_to_center = "descendant"`、offset `> 0`。 |
| `biological_ancestor` | biological ancestor | 血縁上の祖先 / 生物学上の祖先 | 例にあるもの | `extended` | `biological_parent` edge だけを辿る。parentage subtype が必要。 |
| `biological_descendant` | biological descendant | 血縁上の子孫 / 生物学上の子孫 | 逆向き | `extended` | `biological_parent` lineage の逆向き。 |
| `legal_ancestor` | legal ancestor | 法的な祖先 / 法的親族上の祖先 | 例にあるもの。ただし日本語UIでは少し硬い | `extended` | `legal_parent` edge だけを辿る。legal subtype が必要。 |
| `legal_descendant` | legal descendant | 法的な子孫 | 法的親子関係をたどる | `extended` | `legal_parent` lineage の逆向き。 |
| `adoptive_ancestor` | adoptive ancestor | 養親系の祖先 | 養親をたどる | `derived` | `adoptive_parent` edge を含む lineage path から導出。 |
| `adoptive_descendant` | adoptive descendant | 養子系の子孫 | 養子をたどる | `derived` | `adoptive_parent` edge を含む lineage path の逆向き。 |
| `step_ancestor` | step ancestor | 継親系の祖先 | 家系図では通常は補助的 | `extended` | `step_parent` を lineage traversal に含める policy が必要。 |
| `step_descendant` | step descendant | 継子系の子孫 | 同上 | `extended` | `step_parent` traversal の逆向き。 |
| `foster_ancestor` | foster ancestor | 里親系の祖先 | 里親関係をたどる | `extended` | `foster_parent` edge と traversal policy が必要。 |
| `foster_descendant` | foster descendant | 里子系の子孫 | 同上 | `extended` | `foster_parent` traversal の逆向き。 |
| `direct_ancestor` | direct ancestor | 直系尊属 / 直系の祖先 | 親、祖父母、曽祖父母など | `derived` | lineage path で center より上。`relation_to_center = "ancestor"`、offset `< 0`。 |
| `direct_descendant` | direct descendant | 直系卑属 / 直系の子孫 | 子、孫、ひ孫など | `derived` | lineage path で center より下。`relation_to_center = "descendant"`、offset `> 0`。 |
| `collateral_relative` | collateral relative | 傍系親族 | 兄弟姉妹、おじ・おば、いとこなど | `derived` | 共通 ancestor を持つが直系でない path。 |
| `lineal_relative` | lineal relative | 直系親族 | 直系尊属・直系卑属 | `derived` | ancestor / descendant の総称。 |

5. 世代別の直系続柄

| key候補 | 英名 | 和名 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- |
| `father` | father | 父 | `display` | `parent` + node gender male。 |
| `mother` | mother | 母 | `display` | `parent` + node gender female。 |
| `son` | son | 息子 | `display` | `child` + node gender male。 |
| `daughter` | daughter | 娘 | `display` | `child` + node gender female。 |
| `grandparent` | grandparent | 祖父母 | `derived` | ancestor の `generation_offset_from_center` で表現。 |
| `grandfather` | grandfather | 祖父 | `display` | ancestor offset `-2` + node gender male。 |
| `grandmother` | grandmother | 祖母 | `display` | ancestor offset `-2` + node gender female。 |
| `grandchild` | grandchild | 孫 | `derived` | descendant の `generation_offset_from_center` で表現。 |
| `grandson` | grandson | 孫息子 | `display` | descendant offset `2` + node gender male。 |
| `granddaughter` | granddaughter | 孫娘 | `display` | descendant offset `2` + node gender female。 |
| `great_grandparent` | great-grandparent | 曽祖父母 | `derived` | ancestor の `generation_offset_from_center` で表現。 |
| `great_grandfather` | great-grandfather | 曽祖父 | `display` | ancestor offset `-3` + node gender male。 |
| `great_grandmother` | great-grandmother | 曽祖母 | `display` | ancestor offset `-3` + node gender female。 |
| `great_grandchild` | great-grandchild | ひ孫 | `derived` | descendant の `generation_offset_from_center` で表現。 |
| `great_grandson` | great-grandson | ひ孫息子 | `display` | descendant offset `3` + node gender male。 |
| `great_granddaughter` | great-granddaughter | ひ孫娘 | `display` | descendant offset `3` + node gender female。 |
| `great_great_grandparent` | great-great-grandparent | 高祖父母 | `derived` | ancestor の `generation_offset_from_center` で表現。 |
| `great_great_grandfather` | great-great-grandfather | 高祖父 | `display` | ancestor offset `-4` + node gender male。 |
| `great_great_grandmother` | great-great-grandmother | 高祖母 | `display` | ancestor offset `-4` + node gender female。 |
| `great_great_grandchild` | great-great-grandchild | 玄孫 | `derived` | descendant の `generation_offset_from_center` で表現。 |
| `nth_great_grandparent` | nth great-grandparent | n代前の祖先 | `derived` | ancestor の `generation_offset_from_center` で表現。 |
| `nth_great_grandchild` | nth great-grandchild | n代後の子孫 | `derived` | descendant の `generation_offset_from_center` で表現。 |

6. 父方・母方の直系続柄

| key候補 | 英名 | 和名 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- |
| `paternal_grandparent` | paternal grandparent | 父方の祖父母 | `derived` | center からの最初の親 step が father である lineage path。 |
| `paternal_grandfather` | paternal grandfather | 父方の祖父 | `display` | first step が father、offset `-2`、node gender male。 |
| `paternal_grandmother` | paternal grandmother | 父方の祖母 | `display` | first step が father、offset `-2`、node gender female。 |
| `maternal_grandparent` | maternal grandparent | 母方の祖父母 | `derived` | center からの最初の親 step が mother である lineage path。 |
| `maternal_grandfather` | maternal grandfather | 母方の祖父 | `display` | first step が mother、offset `-2`、node gender male。 |
| `maternal_grandmother` | maternal grandmother | 母方の祖母 | `display` | first step が mother、offset `-2`、node gender female。 |
| `paternal_ancestor` | paternal ancestor | 父方の祖先 | `derived` | center からの最初の親 step が father である ancestor path。 |
| `maternal_ancestor` | maternal ancestor | 母方の祖先 | `derived` | center からの最初の親 step が mother である ancestor path。 |
| `paternal_line` | paternal line | 父系 | `derived` | center からの最初の親 step が father である lineage path。 |
| `maternal_line` | maternal line | 母系 | `derived` | center からの最初の親 step が mother である lineage path。 |

7. おじ・おば・甥姪系

| key候補 | 英名 | 和名 | 備考 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- | --- |
| `uncle_or_aunt` | uncle or aunt | おじ・おば | 汎用 | `derived` | parent -> sibling path、または grandparent descendant path。 |
| `uncle` | uncle | おじ | 表示用 | `display` | parent -> sibling path + node gender male。 |
| `aunt` | aunt | おば | 表示用 | `display` | parent -> sibling path + node gender female。 |
| `paternal_uncle` | paternal uncle | 父方のおじ | 日本語の伯父・叔父は年齢差も絡む | `display` | father -> sibling path + node gender male。 |
| `paternal_aunt` | paternal aunt | 父方のおば | 同上 | `display` | father -> sibling path + node gender female。 |
| `maternal_uncle` | maternal uncle | 母方のおじ | 同上 | `display` | mother -> sibling path + node gender male。 |
| `maternal_aunt` | maternal aunt | 母方のおば | 同上 | `display` | mother -> sibling path + node gender female。 |
| `older_uncle` | older uncle | 伯父 | 親より年上の男性きょうだい | `display` | parent -> sibling + node gender + parent との birth_date 比較。 |
| `younger_uncle` | younger uncle | 叔父 | 親より年下の男性きょうだい | `display` | parent -> sibling + node gender + parent との birth_date 比較。 |
| `older_aunt` | older aunt | 伯母 | 親より年上の女性きょうだい | `display` | parent -> sibling + node gender + parent との birth_date 比較。 |
| `younger_aunt` | younger aunt | 叔母 | 親より年下の女性きょうだい | `display` | parent -> sibling + node gender + parent との birth_date 比較。 |
| `great_uncle_or_aunt` | great-uncle or great-aunt | 大おじ・大おば | 祖父母の兄弟姉妹 | `derived` | grandparent -> sibling path。性別は node gender。 |
| `great_uncle` | great-uncle | 大おじ |  | `display` | grandparent -> sibling path。性別は node gender。 |
| `great_aunt` | great-aunt | 大おば |  | `display` | grandparent -> sibling path。性別は node gender。 |
| `nibling` | nibling | 甥姪 | nephew/nieceの中立語 | `derived` | sibling -> child/descendant path。 |
| `nephew` | nephew | 甥 |  | `display` | nibling 系 path + node gender から表示。 |
| `niece` | niece | 姪 |  | `display` | nibling 系 path + node gender から表示。 |
| `grand_nephew_or_niece` | grandnephew or grandniece | 大甥・大姪 |  | `derived` | sibling -> child/descendant path。 |
| `grand_nephew` | grandnephew | 大甥 |  | `display` | nibling 系 path + node gender から表示。 |
| `grand_niece` | grandniece | 大姪 |  | `display` | nibling 系 path + node gender から表示。 |

8. いとこ系

| key候補 | 英名 | 和名 | 備考 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- | --- |
| `cousin` | cousin | いとこ | 汎用 | `derived` | 最小共通 ancestor の depth と世代差から導出。 |
| `first_cousin` | first cousin | いとこ | 祖父母を共有 | `derived` | 最小共通 ancestor の depth と世代差から導出。 |
| `second_cousin` | second cousin | はとこ / またいとこ | 曽祖父母を共有 | `derived` | 最小共通 ancestor の depth と世代差から導出。 |
| `third_cousin` | third cousin | 三いとこ | 高祖父母を共有 | `derived` | 最小共通 ancestor の depth と世代差から導出。 |
| `cousin_once_removed` | cousin once removed | 一代違いのいとこ | 親のいとこ、いとこの子など | `derived` | 最小共通 ancestor の depth と世代差から導出。 |
| `cousin_twice_removed` | cousin twice removed | 二代違いのいとこ | 世代差2 | `derived` | 最小共通 ancestor の depth と世代差から導出。 |
| `paternal_cousin` | paternal cousin | 父方のいとこ | 父方経由 | `derived` | center からの最初の親 step が father の cousin path。 |
| `maternal_cousin` | maternal cousin | 母方のいとこ | 母方経由 | `derived` | center からの最初の親 step が mother の cousin path。 |
| `parallel_cousin` | parallel cousin | 平行いとこ | 親の同性きょうだいの子。文化人類学寄り | `display` | 親とその sibling の gender 関係から表示。文化人類学ラベルとして扱う。 |
| `cross_cousin` | cross cousin | 交差いとこ | 親の異性きょうだいの子。文化人類学寄り | `display` | 親とその sibling の gender 関係から表示。文化人類学ラベルとして扱う。 |

9. 姻族・義理の続柄

| key候補 | 英名 | 和名 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- |
| `in_law` | in-law | 姻族 / 義理の親族 | `derived` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `parent_in_law` | parent-in-law | 義父母 | `derived` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `father_in_law` | father-in-law | 義父 | `display` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `mother_in_law` | mother-in-law | 義母 | `display` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `child_in_law` | child-in-law | 義理の子 | `derived` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `son_in_law` | son-in-law | 娘婿 / 義理の息子 | `display` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `daughter_in_law` | daughter-in-law | 嫁 / 義理の娘 | `display` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `sibling_in_law` | sibling-in-law | 義兄弟姉妹 | `derived` | spouse -> sibling または sibling -> spouse path。性別・年齢は表示派生。 |
| `brother_in_law` | brother-in-law | 義兄 / 義弟 | `display` | spouse -> sibling または sibling -> spouse path。性別・年齢は表示派生。 |
| `sister_in_law` | sister-in-law | 義姉 / 義妹 | `display` | spouse -> sibling または sibling -> spouse path。性別・年齢は表示派生。 |
| `grandparent_in_law` | grandparent-in-law | 義祖父母 | `derived` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `grandfather_in_law` | grandfather-in-law | 義祖父 | `display` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `grandmother_in_law` | grandmother-in-law | 義祖母 | `display` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `grandchild_in_law` | grandchild-in-law | 義理の孫 | `derived` | `spouse` edge を含む path から導出。性別は表示派生。 |
| `uncle_in_law` | uncle-in-law | 義理のおじ | `derived` | spouse -> uncle/aunt または uncle/aunt -> spouse path。 |
| `aunt_in_law` | aunt-in-law | 義理のおば | `derived` | spouse -> uncle/aunt または uncle/aunt -> spouse path。 |
| `cousin_in_law` | cousin-in-law | 義理のいとこ | `derived` | spouse -> cousin または cousin -> spouse path。 |

10. 再婚・連れ子・ステップファミリー系

| key候補 | 英名 | 和名 | 備考 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- | --- |
| `step_family_member` | step family member | 継家族の一員 | 汎用 | `display` | `step_parent` または spouse-child path から導出。深い step relation は traversal policy が必要。 |
| `step_father` | stepfather | 継父 |  | `display` | `step_parent` または spouse-child path から導出。深い step relation は traversal policy が必要。 |
| `step_mother` | stepmother | 継母 |  | `display` | `step_parent` または spouse-child path から導出。深い step relation は traversal policy が必要。 |
| `step_son` | stepson | 継息子 |  | `display` | `step_parent` または spouse-child path から導出。深い step relation は traversal policy が必要。 |
| `step_daughter` | stepdaughter | 継娘 |  | `display` | `step_parent` または spouse-child path から導出。深い step relation は traversal policy が必要。 |
| `step_grandparent` | step-grandparent | 継祖父母 |  | `extended` | `step_parent` または spouse-child path から導出。深い step relation は traversal policy が必要。 |
| `step_grandchild` | step-grandchild | 継孫 |  | `extended` | `step_parent` または spouse-child path から導出。深い step relation は traversal policy が必要。 |
| `step_uncle` | step-uncle | 継おじ |  | `extended` | step_parent の sibling path、または parent spouse の sibling path。 |
| `step_aunt` | step-aunt | 継おば |  | `extended` | step_parent の sibling path、または parent spouse の sibling path。 |
| `step_cousin` | step-cousin | 継いとこ |  | `extended` | step_parent 側の cousin path。step traversal policy が必要。 |
| `parents_spouse` | parent’s spouse | 親の配偶者 | 継親の元データとして有用 | `derived` | parent -> spouse path。step relation の導出元。 |
| `spouses_child` | spouse’s child | 配偶者の子 | 継子の元データとして有用 | `derived` | spouse -> child path。step relation の導出元。 |

11. 養子・里親・後見系

| key候補 | 英名 | 和名 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- |
| `adoptive_family_member` | adoptive family member | 養家族 | `display` | `adoptive_parent` path と node gender / generation offset から導出・表示。 |
| `adoptive_father` | adoptive father | 養父 | `display` | `adoptive_parent` path と node gender / generation offset から導出・表示。 |
| `adoptive_mother` | adoptive mother | 養母 | `display` | `adoptive_parent` path と node gender / generation offset から導出・表示。 |
| `adopted_son` | adopted son | 養子 / 養息子 | `display` | `adoptive_parent` path と node gender / generation offset から導出・表示。 |
| `adopted_daughter` | adopted daughter | 養女 | `display` | `adoptive_parent` path と node gender / generation offset から導出・表示。 |
| `adoptive_grandparent` | adoptive grandparent | 養祖父母 | `derived` | `adoptive_parent` path と node gender / generation offset から導出・表示。 |
| `adoptive_grandchild` | adoptive grandchild | 養孫 | `derived` | `adoptive_parent` path と node gender / generation offset から導出・表示。 |
| `foster_father` | foster father | 里父 | `display` | `foster_parent` edge があれば path と node gender から導出・表示。 |
| `foster_mother` | foster mother | 里母 | `display` | `foster_parent` edge があれば path と node gender から導出・表示。 |
| `foster_son` | foster son | 里子 / 里息子 | `display` | `foster_parent` edge があれば path と node gender から導出・表示。 |
| `foster_daughter` | foster daughter | 里子 / 里娘 | `display` | `foster_parent` edge があれば path と node gender から導出・表示。 |
| `legal_guardian` | legal guardian | 法定後見人 | `extended` | `guardian` edge + legal/temporary status metadata。 |
| `temporary_guardian` | temporary guardian | 一時的な保護者 | `extended` | `guardian` edge + legal/temporary status metadata。 |

12. 家系図表示・計算用の抽象ラベル

| key候補 | 英名 | 和名 | 用途 | 対応目安 | 表現方法 |
| --- | --- | --- | --- | --- | --- |
| `relative` | relative | 親族 | 汎用 | `derived` | graph path が存在し、非親族 edge だけではない場合の総称。 |
| `family_member` | family member | 家族 | 汎用 | `display` | product policy 依存。血縁・法的・同居を含めるかを UI 側で決める。 |
| `household_member` | household member | 世帯員 | 同居・住民票的 | `out-of-scope` | genealogy graph ではなく household / residence model 推奨。 |
| `blood_relative` | blood relative | 血縁者 | biological 系 | `extended` | biological/legal parentage subtype が必要。 |
| `legal_relative` | legal relative | 法的親族 | legal 系 | `extended` | biological/legal parentage subtype が必要。 |
| `affinal_relative` | affinal relative | 姻族 | 婚姻による親族 | `derived` | `spouse` edge を含む path。 |
| `lineal_ascendant` | lineal ascendant | 直系尊属 | 親・祖父母など | `derived` | lineage path で center より上。`relation_to_center = "ancestor"`、offset `< 0`。 |
| `lineal_descendant` | lineal descendant | 直系卑属 | 子・孫など | `derived` | lineage path で center より下。`relation_to_center = "descendant"`、offset `> 0`。 |
| `collateral_kin` | collateral kin | 傍系親族 | 兄弟姉妹・いとこなど | `derived` | 共通 ancestor を持つが直系でない path。 |
| `kinship` | kinship | 続柄 / 親族関係 | 概念名 | `display` | API field ではなく概念名。storage / UI 用語として扱う。 |
| `relationship` | relationship | 関係性 | DB概念名 | `display` | API field ではなく概念名。storage / UI 用語として扱う。 |
