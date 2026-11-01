// GENERATED from plugins/basic-set-sample/data/skills.toml and
// plugins/lang-ru/locales/ru/basic-set-skills.ftl for the browser preview backend.
// Own-words texts, not book text; page references are unverified.
import type { SkillDef, TechniqueDef, TraitDef } from './mockTypes';

export const SKILL_DEFS: SkillDef[] = [
  {
    "id": "broadsword",
    "name": "Broadsword",
    "attribute": "DX",
    "difficulty": "A",
    "page": "B208",
    "defaults": [
      {
        "attribute": "DX",
        "modifier": -5
      },
      {
        "skill": "shortsword",
        "modifier": -2
      }
    ],
    "description": "Fighting with a balanced one-handed blade of roughly arm's length: arming swords, sabres, heavy batons and similar weapons. The skill covers attacking, parrying and the footwork that goes with them.\n\nIt is the bread-and-butter weapon skill of knights, guards and adventurers. Untrained people can still swing a sword at a heavy penalty, and anyone who knows a related blade skill does noticeably better than a complete novice.\n"
  },
  {
    "id": "shield",
    "name": "Shield",
    "attribute": "DX",
    "difficulty": "E",
    "page": "B220",
    "defaults": [
      {
        "attribute": "DX",
        "modifier": -4
      }
    ],
    "description": "Using a shield to block blows and missiles, and to keep it between yourself and the enemy while you fight with the other hand. Higher skill means more reliable blocks.\n\nAny fighter who carries a shield should have at least a point or two here. The shield's own Defence Bonus comes from the equipment: the sheet shows Block from this skill and the DB of a carried shield.\n"
  },
  {
    "id": "brawling",
    "name": "Brawling",
    "attribute": "DX",
    "difficulty": "E",
    "page": "B182",
    "defaults": [],
    "description": "Rough, untrained-looking but effective unarmed fighting: punches, elbows, headbutts and dirty tricks learned in taverns, alleys and barracks. It improves hitting with bare hands and parrying without a weapon.\n\nIt is cheap and useful for almost anyone who might end up in a scuffle. Very skilled brawlers also hit harder: at DX+2 or better the sheet adds +1 damage per die to punches and kicks.\n"
  },
  {
    "id": "knife",
    "name": "Knife",
    "attribute": "DX",
    "difficulty": "E",
    "page": "B208",
    "defaults": [
      {
        "attribute": "DX",
        "modifier": -4
      }
    ],
    "description": "Fighting with a short blade held in the hand: daggers, knives and similar. The skill covers quick stabs and slashes at close range and parrying with the blade, which is harder than with a longer weapon.\n\nThieves, sailors, assassins and anyone who carries a knife for work or self-defence tend to know it. Throwing a knife is a separate skill.\n"
  },
  {
    "id": "survival",
    "name": "Survival",
    "attribute": "Per",
    "difficulty": "A",
    "page": "B223",
    "defaults": [
      {
        "attribute": "Per",
        "modifier": -5
      },
      {
        "skill": "naturalist",
        "modifier": -3
      }
    ],
    "specialization_required": true,
    "specializations": [
      "Woodlands",
      "Desert",
      "Mountain",
      "Arctic",
      "Jungle",
      "Plains",
      "Swampland",
      "Island/Beach"
    ],
    "description": "Living off the land in one kind of terrain: finding water and food, building a shelter, making fire and avoiding natural dangers. You must pick the terrain, such as woodlands, desert or mountains, and the skill is much weaker elsewhere.\n\nScouts, hunters, rangers and castaways rely on it. Write the terrain in the specialization field, for example Woodlands.\n"
  },
  {
    "id": "tracking",
    "name": "Tracking",
    "attribute": "Per",
    "difficulty": "A",
    "page": "B226",
    "defaults": [
      {
        "attribute": "Per",
        "modifier": -5
      },
      {
        "skill": "naturalist",
        "modifier": -5
      }
    ],
    "description": "Following the trail of people or animals by reading footprints, broken twigs, disturbed ground and similar signs. Fresh, clear trails are easy; old, rained-on or deliberately hidden ones are much harder.\n\nHunters, scouts and bounty hunters use it constantly. A quarry that knows how to hide its tracks can make the job considerably more difficult.\n"
  },
  {
    "id": "stealth",
    "name": "Stealth",
    "attribute": "DX",
    "difficulty": "A",
    "page": "B222",
    "defaults": [
      {
        "attribute": "DX",
        "modifier": -5
      },
      {
        "attribute": "IQ",
        "modifier": -5
      }
    ],
    "description": "Moving quietly and staying out of sight: sneaking past guards, hiding in shadows, following someone without being noticed. Heavy loads and noisy armour make it harder.\n\nAlmost every thief, scout and spy needs it. Anyone can try to sneak at default, but trained characters are far more reliable.\n"
  },
  {
    "id": "climbing",
    "name": "Climbing",
    "attribute": "DX",
    "difficulty": "A",
    "page": "B183",
    "defaults": [
      {
        "attribute": "DX",
        "modifier": -5
      }
    ],
    "description": "Climbing trees, walls, cliffs and ropes, and knowing how to pick a safe route. Easy climbs with good holds need only occasional rolls; sheer or slippery surfaces are much harder.\n\nBurglars, mountaineers, sailors and scouts find it valuable. A heavy load slows climbing down and makes falls more likely.\n"
  },
  {
    "id": "first-aid",
    "name": "First Aid",
    "attribute": "IQ",
    "difficulty": "E",
    "page": "B195",
    "defaults": [
      {
        "attribute": "IQ",
        "modifier": -4
      }
    ],
    "description": "Basic emergency care: stopping bleeding, bandaging wounds, treating shock and keeping someone alive until real medical help arrives. It restores a little lost health right after an injury.\n\nUseful for every adventuring group; one character with this skill can make a big difference after a fight. It is not a replacement for a trained physician.\n"
  },
  {
    "id": "swimming",
    "name": "Swimming",
    "attribute": "HT",
    "difficulty": "E",
    "page": "B224",
    "defaults": [
      {
        "attribute": "HT",
        "modifier": -4
      }
    ],
    "description": "Staying afloat and moving through water, including diving in and rescuing someone who is drowning. Calm water is easy; currents, waves, cold and heavy gear make it dangerous.\n\nSailors, river folk and anyone who travels by boat should consider it. Armour and a full pack can make even a good swimmer sink.\n"
  },
  {
    "id": "naturalist",
    "name": "Naturalist",
    "attribute": "IQ",
    "difficulty": "H",
    "page": "B211",
    "defaults": [
      {
        "attribute": "IQ",
        "modifier": -6
      }
    ],
    "description": "Practical knowledge of the living world: which plants are edible or poisonous, how animals behave, how to read the weather and the seasons. It is broader and more theoretical than survival in one terrain.\n\nDruids, hunters, herbalists and explorers benefit from it. Several outdoor skills can fall back on it when the character has no direct training.\n"
  }
];

export const TECHNIQUE_DEFS: TechniqueDef[] = [
  {
    "id": "kicking",
    "name": "Kicking",
    "skill": "brawling",
    "difficulty": "H",
    "modifier": -2,
    "max": 0,
    "page": "B231",
    "description": "A trained kick: a strong blow with the foot that hits harder than a punch but is riskier, since a miss can leave you off balance. It starts from your unarmed skill at a penalty and can be bought up to that skill's level.\n\nBrawlers and martial artists like it as a heavy hitting option. The Kick attack in the weapons table uses this level when the technique is bought.\n"
  },
  {
    "id": "off-hand-weapon-training",
    "name": "Off-Hand Weapon Training",
    "skill": "broadsword",
    "difficulty": "H",
    "modifier": -4,
    "max": 0,
    "page": "B232",
    "description": "Practice with a weapon in the off hand, which normally suffers a heavy penalty. Each level removes part of that penalty, up to fighting as well with the off hand as with the main one.\n\nTwo-weapon fighters and duellists who expect to lose the use of their good arm buy it. Take it separately for each weapon skill.\n"
  }
];

export const TALENT_DEF: TraitDef = {
  "id": "outdoorsman",
  "name": "Outdoorsman",
  "category": "advantage",
  "cost_per_level": 10,
  "max_levels": 4,
  "page": "B90",
  "features": [
    {
      "type": "skill_bonus",
      "skills": [
        "camouflage",
        "fishing",
        "mimicry",
        "naturalist",
        "navigation",
        "survival",
        "tracking",
        "weather-sense"
      ],
      "amount": 1,
      "per_level": true
    }
  ],
  "description": "A natural feel for the wilderness. Each level adds +1 to a group of outdoor skills: camouflage, fishing, mimicry, naturalist, navigation, survival, tracking and weather sense. The bonus is applied automatically to those skills on the sheet.\n\nIt suits rangers, hunters and anyone raised far from towns. A talent is usually cheaper than raising every one of those skills separately if the character uses several of them.\n",
  "cost": 0
};

export const SKILL_CATALOG_RU: Record<string, { name: string; description: string }> = {
  "skill-broadsword": {
    "name": "Палаш",
    "description": "Бой сбалансированным одноручным клинком длиной примерно в руку: мечами, саблями, тяжёлыми дубинками и похожим оружием. Навык охватывает атаки, парирование и сопутствующую работу ног.\n\nЭто основной оружейный навык рыцарей, стражников и искателей приключений. Необученный человек тоже может махать мечом, но с большим штрафом, а владеющий родственным клинковым навыком справится заметно лучше новичка."
  },
  "skill-shield": {
    "name": "Щит",
    "description": "Умение подставлять щит под удары и снаряды и держать его между собой и противником, пока другая рука сражается. Чем выше навык, тем надёжнее блок.\n\nЛюбому бойцу со щитом стоит вложить сюда хотя бы пару очков. Защитный бонус (DB) щита — свойство снаряжения: лист показывает блок по этому навыку и DB носимого щита."
  },
  "skill-brawling": {
    "name": "Драка",
    "description": "Грубый, на вид неучёный, но действенный рукопашный бой: удары кулаком, локтем, головой и грязные приёмы из таверн, переулков и казарм. Повышает шанс попасть голыми руками и парировать без оружия.\n\nДёшев и полезен почти всем, кто может ввязаться в потасовку. Очень умелые драчуны бьют ещё и сильнее; при навыке не ниже ЛВ+2 лист добавляет +1 к урону за кубик ударам рукой и ногой."
  },
  "skill-knife": {
    "name": "Нож",
    "description": "Бой коротким клинком в руке: кинжалами, ножами и подобным. Навык охватывает быстрые уколы и порезы вблизи и парирование клинком, которое даётся труднее, чем более длинным оружием.\n\nЕго знают воры, моряки, убийцы и все, кто носит нож для работы или самозащиты. Метание ножа — отдельный навык."
  },
  "skill-survival": {
    "name": "Выживание",
    "description": "Умение прожить за счёт природы в одном типе местности: найти воду и пищу, построить укрытие, развести огонь и избежать природных опасностей. Местность нужно выбрать — например, лес, пустыню или горы; в других краях навык работает гораздо хуже.\n\nНа него полагаются разведчики, охотники, следопыты и потерпевшие кораблекрушение. Местность впишите в поле специализации, например Woodlands (лес)."
  },
  "skill-tracking": {
    "name": "Выслеживание",
    "description": "Умение идти по следу людей или животных, читая отпечатки, сломанные ветки, примятую землю и прочие знаки. Свежий чёткий след найти легко, старый, размытый дождём или нарочно скрытый — гораздо труднее.\n\nОхотники, разведчики и охотники за головами пользуются им постоянно. Добыча, умеющая заметать следы, может сильно усложнить задачу."
  },
  "skill-stealth": {
    "name": "Скрытность",
    "description": "Умение двигаться тихо и оставаться незамеченным: прокрасться мимо стражи, спрятаться в тени, проследить за кем-то. Тяжёлая ноша и шумная броня мешают.\n\nНужна почти каждому вору, разведчику и шпиону. Красться по умолчанию может любой, но обученный персонаж справляется гораздо надёжнее."
  },
  "skill-climbing": {
    "name": "Лазание",
    "description": "Умение лазать по деревьям, стенам, скалам и верёвкам и выбирать безопасный путь. Лёгкий подъём с хорошими зацепами требует редких бросков, отвесные или скользкие поверхности — гораздо сложнее.\n\nЦенен для взломщиков, альпинистов, моряков и разведчиков. Тяжёлая ноша замедляет подъём и повышает риск падения."
  },
  "skill-first-aid": {
    "name": "Первая помощь",
    "description": "Простейшая неотложная помощь: остановить кровь, перевязать рану, справиться с шоком и дотянуть пострадавшего до настоящего лечения. Сразу после ранения возвращает немного потерянного здоровья.\n\nПолезна любой группе искателей приключений: один персонаж с этим навыком сильно меняет исход после боя. Полноценного врача он не заменяет."
  },
  "skill-swimming": {
    "name": "Плавание",
    "description": "Умение держаться на воде и передвигаться в ней, нырять и спасать утопающих. В спокойной воде это просто, течение, волны, холод и тяжёлое снаряжение делают плавание опасным.\n\nСтоит взять морякам, речным жителям и всем, кто путешествует по воде. Броня и полный рюкзак могут утопить даже хорошего пловца."
  },
  "skill-naturalist": {
    "name": "Натуралист",
    "description": "Практическое знание живой природы: какие растения съедобны или ядовиты, как ведут себя животные, как читать погоду и времена года. Шире и теоретичнее, чем выживание в одной местности.\n\nПолезен друидам, охотникам, травникам и исследователям. Многие навыки дикой местности могут опираться на него, когда прямой выучки нет."
  },
  "technique-kicking": {
    "name": "Пинок",
    "description": "Отработанный удар ногой: сильнее удара кулаком, но рискованнее — промах может лишить равновесия. Начинается от навыка рукопашного боя со штрафом и может быть доведён до уровня этого навыка.\n\nДрачуны и мастера единоборств ценят его как тяжёлый удар. Атака «Удар ногой» в таблице оружия использует этот уровень, если приём куплен."
  },
  "technique-off-hand-weapon-training": {
    "name": "Владение оружием в другой руке",
    "description": "Тренировка оружия в неосновной руке, которая обычно получает большой штраф. Каждый уровень снимает часть штрафа, вплоть до того, что неосновная рука сражается не хуже основной.\n\nБерут бойцы с двумя клинками и дуэлянты, опасающиеся остаться без рабочей руки. Покупается отдельно для каждого оружейного навыка."
  },
  "trait-outdoorsman": {
    "name": "Знаток дикой природы",
    "description": "Врождённое чутьё на дикую природу. Каждый уровень даёт +1 к группе навыков дикой местности: маскировке, рыбалке, имитации голосов, натуралисту, навигации, выживанию, выслеживанию и чувству погоды. Бонус автоматически прибавляется к этим навыкам на листе.\n\nПодходит следопытам, охотникам и всем, кто вырос вдали от городов. Если персонаж пользуется несколькими такими навыками, талант обычно дешевле, чем поднимать каждый отдельно."
  }
};
