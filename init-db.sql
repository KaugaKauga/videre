-- Videre Test Database Initialization Script
-- Greek Mythology themed database for testing
-- Enhanced version with corrections and additions

-- Primordial Deities table (First Generation)
CREATE TABLE primordials (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    gender VARCHAR(20) CHECK (gender IN ('male', 'female', 'non-binary', 'unknown')),
    domain VARCHAR(100) NOT NULL,
    symbol VARCHAR(100),
    realm VARCHAR(50),
    power_level INTEGER CHECK (power_level >= 1 AND power_level <= 10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Titans table (Second Generation)
CREATE TABLE titans (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    roman_name VARCHAR(100),
    gender VARCHAR(20) CHECK (gender IN ('male', 'female', 'non-binary', 'unknown')),
    domain VARCHAR(100) NOT NULL,
    symbol VARCHAR(100),
    realm VARCHAR(50),
    parent_primordial_id INTEGER REFERENCES primordials(id),
    parent_titan_id INTEGER, -- Self-reference added later via ALTER
    spouse_titan_id INTEGER, -- Self-reference added later via ALTER
    power_level INTEGER CHECK (power_level >= 1 AND power_level <= 10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add self-referential foreign keys after table creation
ALTER TABLE titans ADD CONSTRAINT fk_parent_titan FOREIGN KEY (parent_titan_id) REFERENCES titans(id);
ALTER TABLE titans ADD CONSTRAINT fk_spouse_titan FOREIGN KEY (spouse_titan_id) REFERENCES titans(id);

-- Gods and Goddesses table (Olympians and their descendants)
CREATE TABLE gods (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    roman_name VARCHAR(100),
    gender VARCHAR(20) CHECK (gender IN ('male', 'female', 'non-binary', 'unknown')),
    domain VARCHAR(100) NOT NULL,
    symbol VARCHAR(100),
    realm VARCHAR(50),
    parent_id INTEGER REFERENCES gods(id),
    parent_titan_id INTEGER REFERENCES titans(id),
    second_parent_id INTEGER REFERENCES gods(id),
    second_parent_titan_id INTEGER REFERENCES titans(id),
    spouse_id INTEGER, -- Self-reference added later via ALTER
    is_olympian BOOLEAN DEFAULT false,
    power_level INTEGER CHECK (power_level >= 1 AND power_level <= 10),
    birth_method VARCHAR(100), -- e.g., 'normal', 'from_head', 'parthenogenesis', 'from_sea_foam'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add self-referential foreign key for spouse
ALTER TABLE gods ADD CONSTRAINT fk_spouse_god FOREIGN KEY (spouse_id) REFERENCES gods(id);

-- Heroes table
CREATE TABLE heroes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    gender VARCHAR(20) CHECK (gender IN ('male', 'female', 'non-binary', 'unknown')) DEFAULT 'male',
    birth_place VARCHAR(100),
    patron_god_id INTEGER REFERENCES gods(id),
    is_demigod BOOLEAN DEFAULT false,
    mortal_parent VARCHAR(100),
    divine_parent_id INTEGER REFERENCES gods(id),
    fame_level INTEGER CHECK (fame_level >= 1 AND fame_level <= 10),
    status VARCHAR(50) DEFAULT 'mortal',
    weapon VARCHAR(100),
    special_ability TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Monsters and Creatures table
CREATE TABLE creatures (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    type VARCHAR(50) NOT NULL,
    description TEXT,
    homeland VARCHAR(100),
    parent_creature_id INTEGER REFERENCES creatures(id),
    threat_level INTEGER CHECK (threat_level >= 1 AND threat_level <= 10),
    is_immortal BOOLEAN DEFAULT false,
    weakness VARCHAR(200),
    slain_by_hero_id INTEGER REFERENCES heroes(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Quests and Labors table
CREATE TABLE quests (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    hero_id INTEGER REFERENCES heroes(id),
    quest_type VARCHAR(50),
    difficulty INTEGER CHECK (difficulty >= 1 AND difficulty <= 10),
    location VARCHAR(100),
    objective TEXT NOT NULL,
    reward TEXT,
    status VARCHAR(50) DEFAULT 'in_progress' CHECK (status IN ('in_progress', 'completed', 'failed', 'abandoned')),
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Legendary Artifacts table
CREATE TABLE artifacts (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    type VARCHAR(50) NOT NULL,
    forged_by_god_id INTEGER REFERENCES gods(id),
    forged_by_titan_id INTEGER REFERENCES titans(id),
    current_owner_id INTEGER,
    owner_type VARCHAR(20) CHECK (owner_type IN ('god', 'hero', 'creature', 'titan', 'lost')),
    power_description TEXT,
    material VARCHAR(100),
    is_cursed BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Relationships table for complex connections (affairs, rivalries, alliances)
CREATE TABLE relationships (
    id SERIAL PRIMARY KEY,
    entity1_id INTEGER NOT NULL,
    entity1_type VARCHAR(20) CHECK (entity1_type IN ('god', 'titan', 'hero', 'creature', 'primordial')),
    entity2_id INTEGER NOT NULL,
    entity2_type VARCHAR(20) CHECK (entity2_type IN ('god', 'titan', 'hero', 'creature', 'primordial')),
    relationship_type VARCHAR(50) CHECK (relationship_type IN ('spouse', 'lover', 'rival', 'enemy', 'ally', 'mentor', 'servant')),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Insert Primordial Deities (Born from Chaos)
INSERT INTO primordials (name, gender, domain, symbol, realm, power_level) VALUES
    ('Chaos', 'unknown', 'The Void, Nothingness', 'Abyss', 'The Void', 10),
    ('Gaia', 'female', 'Earth, Mother of All', 'Earth, Fertile Soil', 'Earth', 10),
    ('Uranus', 'male', 'Sky, Heavens', 'Starry Sky', 'Sky', 10),
    ('Nyx', 'female', 'Night, Darkness', 'Stars, Black Cloak', 'Tartarus', 9),
    ('Erebus', 'male', 'Darkness, Shadow', 'Mist, Shadows', 'Underworld', 9),
    ('Tartarus', 'male', 'The Abyss, Deepest Pit', 'Prison Chains', 'Deepest Underworld', 9),
    ('Eros (Primordial)', 'male', 'Primordial Love, Procreation', 'None', 'Everywhere', 8),
    ('Pontus', 'male', 'Sea, Father of Sea Creatures', 'Waves', 'Sea', 8),
    ('Aether', 'male', 'Light, Upper Atmosphere', 'Bright Light', 'Upper Sky', 8),
    ('Hemera', 'female', 'Day, Daylight', 'Sun', 'Sky', 7),
    ('Ourea', 'male', 'Mountains', 'Mountain Peaks', 'Mountains', 7),
    ('Ananke', 'female', 'Necessity, Inevitability', 'Spindle', 'Cosmos', 9),
    ('Chronos', 'male', 'Time (Primordial)', 'Serpent, Hourglass', 'Cosmos', 9),
    ('Phanes', 'non-binary', 'Procreation, Life', 'Golden Wings', 'Cosmos', 8);

-- Insert the Twelve Titans (Children of Gaia and Uranus)
INSERT INTO titans (name, roman_name, gender, domain, symbol, realm, power_level, parent_primordial_id) VALUES
    ('Cronus', 'Saturn', 'male', 'Time, King of Titans', 'Sickle, Scythe', 'Mount Othrys', 10, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Rhea', 'Ops', 'female', 'Fertility, Motherhood', 'Lion, Turret Crown', 'Mount Othrys', 9, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Oceanus', NULL, 'male', 'Ocean, World River', 'Serpent, Fish', 'Ocean Stream', 9, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Tethys', NULL, 'female', 'Fresh Water, Nursing', 'Water Jug', 'Ocean', 8, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Hyperion', NULL, 'male', 'Light, Watchfulness', 'Sun', 'Sky', 8, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Theia', NULL, 'female', 'Sight, Brilliance', 'Shining Light', 'Sky', 8, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Coeus', NULL, 'male', 'Intelligence, Inquiry', 'Stars', 'North Pillar', 7, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Phoebe', NULL, 'female', 'Prophecy, Intellect', 'Moon', 'Delphi', 7, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Themis', NULL, 'female', 'Divine Law, Order', 'Scales, Sword', 'Mount Olympus', 8, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Mnemosyne', NULL, 'female', 'Memory, Remembrance', 'Lamp', 'Pieria', 7, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Iapetus', NULL, 'male', 'Mortality, Craftiness', 'Spear', 'West Pillar', 7, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Crius', NULL, 'male', 'Constellations, Heavenly Bodies', 'Ram', 'South Pillar', 7, (SELECT id FROM primordials WHERE name = 'Gaia'));

-- Update Titan spouses
UPDATE titans SET spouse_titan_id = (SELECT id FROM titans WHERE name = 'Rhea') WHERE name = 'Cronus';
UPDATE titans SET spouse_titan_id = (SELECT id FROM titans WHERE name = 'Cronus') WHERE name = 'Rhea';
UPDATE titans SET spouse_titan_id = (SELECT id FROM titans WHERE name = 'Tethys') WHERE name = 'Oceanus';
UPDATE titans SET spouse_titan_id = (SELECT id FROM titans WHERE name = 'Oceanus') WHERE name = 'Tethys';
UPDATE titans SET spouse_titan_id = (SELECT id FROM titans WHERE name = 'Theia') WHERE name = 'Hyperion';
UPDATE titans SET spouse_titan_id = (SELECT id FROM titans WHERE name = 'Hyperion') WHERE name = 'Theia';
UPDATE titans SET spouse_titan_id = (SELECT id FROM titans WHERE name = 'Phoebe') WHERE name = 'Coeus';
UPDATE titans SET spouse_titan_id = (SELECT id FROM titans WHERE name = 'Coeus') WHERE name = 'Phoebe';

-- Insert Second Generation Titans (Children of Titans)
INSERT INTO titans (name, roman_name, gender, domain, symbol, realm, power_level, parent_titan_id) VALUES
    ('Prometheus', NULL, 'male', 'Forethought, Fire Giver', 'Torch, Fennel Staff', 'Earth', 8, (SELECT id FROM titans WHERE name = 'Iapetus')),
    ('Epimetheus', NULL, 'male', 'Afterthought, Excuses', 'None', 'Earth', 6, (SELECT id FROM titans WHERE name = 'Iapetus')),
    ('Atlas', NULL, 'male', 'Endurance, Astronomy', 'Celestial Sphere', 'Edge of World', 9, (SELECT id FROM titans WHERE name = 'Iapetus')),
    ('Menoetius', NULL, 'male', 'Violent Anger, Rash Action', 'None', 'Tartarus', 6, (SELECT id FROM titans WHERE name = 'Iapetus')),
    ('Leto', 'Latona', 'female', 'Motherhood, Modesty', 'Veil, Date Palm', 'Delos', 7, (SELECT id FROM titans WHERE name = 'Coeus')),
    ('Asteria', NULL, 'female', 'Falling Stars, Necromancy', 'Star', 'Sky', 6, (SELECT id FROM titans WHERE name = 'Coeus')),
    ('Metis', NULL, 'female', 'Wisdom, Cunning', 'None', 'Inside Zeus', 8, (SELECT id FROM titans WHERE name = 'Oceanus')),
    ('Styx', NULL, 'female', 'River Styx, Hatred', 'River', 'Underworld', 7, (SELECT id FROM titans WHERE name = 'Oceanus')),
    ('Perses', NULL, 'male', 'Destruction', 'None', 'Earth', 6, (SELECT id FROM titans WHERE name = 'Crius')),
    ('Pallas', NULL, 'male', 'Warcraft', 'Spear', 'Earth', 6, (SELECT id FROM titans WHERE name = 'Crius')),
    ('Astraeus', NULL, 'male', 'Dusk, Stars, Astrology', 'Stars', 'Sky', 6, (SELECT id FROM titans WHERE name = 'Crius')),
    ('Clymene', 'Asia', 'female', 'Fame, Renown', 'None', 'Earth', 5, (SELECT id FROM titans WHERE name = 'Oceanus'));

-- Insert Children of Titans (Pre-Olympian Deities)
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_titan_id, second_parent_titan_id) VALUES
    ('Helios', 'Sol', 'male', 'The Sun, Light of Day', 'Sun Chariot, Crown of Rays', 'Sky', false, 8, (SELECT id FROM titans WHERE name = 'Hyperion'), (SELECT id FROM titans WHERE name = 'Theia')),
    ('Selene', 'Luna', 'female', 'The Moon, Night Light', 'Moon Crescent, Torch', 'Sky', false, 7, (SELECT id FROM titans WHERE name = 'Hyperion'), (SELECT id FROM titans WHERE name = 'Theia')),
    ('Eos', 'Aurora', 'female', 'Dawn, Morning', 'Roses, Saffron Robe', 'Sky', false, 7, (SELECT id FROM titans WHERE name = 'Hyperion'), (SELECT id FROM titans WHERE name = 'Theia'));

INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_titan_id) VALUES
    ('Hecate', 'Trivia', 'female', 'Magic, Crossroads, Necromancy', 'Torch, Keys, Dagger', 'Crossroads', false, 8, (SELECT id FROM titans WHERE name = 'Perses'));

-- Insert the Olympian Gods (Children of Cronus and Rhea)
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_titan_id, second_parent_titan_id) VALUES
    ('Zeus', 'Jupiter', 'male', 'King of Gods, Sky and Thunder', 'Lightning Bolt, Eagle', 'Mount Olympus', true, 10, (SELECT id FROM titans WHERE name = 'Cronus'), (SELECT id FROM titans WHERE name = 'Rhea')),
    ('Hera', 'Juno', 'female', 'Queen of Gods, Marriage', 'Peacock, Cow', 'Mount Olympus', true, 9, (SELECT id FROM titans WHERE name = 'Cronus'), (SELECT id FROM titans WHERE name = 'Rhea')),
    ('Poseidon', 'Neptune', 'male', 'Sea, Earthquakes, Horses', 'Trident, Horse', 'Underwater Palace', true, 10, (SELECT id FROM titans WHERE name = 'Cronus'), (SELECT id FROM titans WHERE name = 'Rhea')),
    ('Demeter', 'Ceres', 'female', 'Agriculture, Harvest', 'Wheat, Cornucopia', 'Fields', true, 8, (SELECT id FROM titans WHERE name = 'Cronus'), (SELECT id FROM titans WHERE name = 'Rhea')),
    ('Hades', 'Pluto', 'male', 'Underworld, Dead, Wealth', 'Helm of Darkness, Cerberus', 'Underworld', false, 10, (SELECT id FROM titans WHERE name = 'Cronus'), (SELECT id FROM titans WHERE name = 'Rhea')),
    ('Hestia', 'Vesta', 'female', 'Hearth, Home, Family', 'Flame, Kettle', 'Mount Olympus', false, 7, (SELECT id FROM titans WHERE name = 'Cronus'), (SELECT id FROM titans WHERE name = 'Rhea'));

-- Update Zeus and Hera as spouses
UPDATE gods SET spouse_id = (SELECT id FROM gods WHERE name = 'Hera') WHERE name = 'Zeus';
UPDATE gods SET spouse_id = (SELECT id FROM gods WHERE name = 'Zeus') WHERE name = 'Hera';

-- Update Hades and Persephone spouses (after Persephone is inserted below)

-- Insert Children of Zeus (with Leto)
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_id, second_parent_titan_id) VALUES
    ('Apollo', 'Phoebus', 'male', 'Sun, Music, Prophecy, Healing, Archery', 'Lyre, Laurel Wreath, Sun', 'Mount Olympus', true, 9, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Leto')),
    ('Artemis', 'Diana', 'female', 'Moon, Hunt, Wilderness, Chastity', 'Bow and Arrow, Deer, Moon', 'Forest', true, 9, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Leto'));

-- Insert Athena (born from Zeus's head after he swallowed Metis)
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_id, birth_method) VALUES
    ('Athena', 'Minerva', 'female', 'Wisdom, Warfare, Crafts', 'Owl, Olive Tree, Aegis', 'Mount Olympus', true, 9, (SELECT id FROM gods WHERE name = 'Zeus'), 'from_head');

-- Insert Children of Zeus and Hera
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_id, second_parent_id) VALUES
    ('Ares', 'Mars', 'male', 'War, Violence, Bloodshed', 'Spear, Vulture, Dog', 'Mount Olympus', true, 8, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM gods WHERE name = 'Hera')),
    ('Hebe', 'Juventas', 'female', 'Youth, Cupbearer', 'Cup, Fountain', 'Mount Olympus', false, 5, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM gods WHERE name = 'Hera')),
    ('Eileithyia', 'Lucina', 'female', 'Childbirth, Labor', 'Torch', 'Mount Olympus', false, 6, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM gods WHERE name = 'Hera'));

-- Insert Hephaestus (parthenogenesis by Hera in main myth)
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_id, birth_method) VALUES
    ('Hephaestus', 'Vulcan', 'male', 'Fire, Forges, Metalworking', 'Hammer, Anvil, Tongs', 'Volcano', true, 8, (SELECT id FROM gods WHERE name = 'Hera'), 'parthenogenesis');

-- Insert other Children of Zeus
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_id) VALUES
    ('Hermes', 'Mercury', 'male', 'Messengers, Trade, Thieves, Travel', 'Winged Sandals, Caduceus', 'Mount Olympus', true, 8, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Dionysus', 'Bacchus', 'male', 'Wine, Festivity, Theatre, Madness', 'Grapes, Thyrsus, Ivy', 'Vineyards', true, 7, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Persephone', 'Proserpina', 'female', 'Spring, Queen of Underworld, Vegetation', 'Pomegranate, Flowers, Torch', 'Underworld', false, 7, (SELECT id FROM gods WHERE name = 'Zeus'));

-- Update second parent for Dionysus and Persephone
UPDATE gods SET second_parent_id = (SELECT id FROM gods WHERE name = 'Demeter') WHERE name = 'Persephone';

-- Update Hades and Persephone as spouses
UPDATE gods SET spouse_id = (SELECT id FROM gods WHERE name = 'Persephone') WHERE name = 'Hades';
UPDATE gods SET spouse_id = (SELECT id FROM gods WHERE name = 'Hades') WHERE name = 'Persephone';

-- Insert Aphrodite (special birth - from sea foam after Uranus was castrated)
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, birth_method) VALUES
    ('Aphrodite', 'Venus', 'female', 'Love, Beauty, Desire', 'Dove, Rose, Myrtle, Scallop Shell', 'Mount Olympus', true, 8, 'from_sea_foam');

-- Update Aphrodite and Hephaestus as spouses (arranged marriage)
UPDATE gods SET spouse_id = (SELECT id FROM gods WHERE name = 'Aphrodite') WHERE name = 'Hephaestus';
UPDATE gods SET spouse_id = (SELECT id FROM gods WHERE name = 'Hephaestus') WHERE name = 'Aphrodite';

-- Insert Sea Deities
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_id) VALUES
    ('Triton', NULL, 'male', 'Sea Messenger, Calm Seas', 'Conch Shell, Trident', 'Underwater Palace', false, 6, (SELECT id FROM gods WHERE name = 'Poseidon')),
    ('Amphitrite', 'Salacia', 'female', 'Queen of the Sea, Sea Life', 'Crab, Seaweed, Trident', 'Underwater Palace', false, 6, NULL),
    ('Thetis', NULL, 'female', 'Sea Nymph, Mother of Achilles', 'Sea Shells, Silver Feet', 'Sea', false, 6, NULL),
    ('Proteus', NULL, 'male', 'Sea, Prophecy, Shape-shifting', 'Seal, Fish', 'Pharos Island', false, 5, (SELECT id FROM gods WHERE name = 'Poseidon'));

-- Update Poseidon and Amphitrite as spouses
UPDATE gods SET spouse_id = (SELECT id FROM gods WHERE name = 'Amphitrite') WHERE name = 'Poseidon';
UPDATE gods SET spouse_id = (SELECT id FROM gods WHERE name = 'Poseidon') WHERE name = 'Amphitrite';

-- Insert Minor Gods and Personifications (Children of various Olympians)
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_id) VALUES
    ('Eros', 'Cupid', 'male', 'Love, Desire, Attraction', 'Bow and Arrow, Wings', 'Mount Olympus', false, 6, (SELECT id FROM gods WHERE name = 'Aphrodite')),
    ('Nike', 'Victoria', 'female', 'Victory, Success', 'Wings, Laurel Wreath', 'Mount Olympus', false, 6, NULL),
    ('Pan', 'Faunus', 'male', 'Nature, Shepherds, Flocks, Rustic Music', 'Pan Flute, Shepherd Staff', 'Arcadia', false, 6, (SELECT id FROM gods WHERE name = 'Hermes')),
    ('Asclepius', 'Vejovis', 'male', 'Medicine, Healing, Rejuvenation', 'Rod of Asclepius, Serpent', 'Epidaurus', false, 7, (SELECT id FROM gods WHERE name = 'Apollo')),
    ('Phobos', 'Timor', 'male', 'Fear, Panic', 'None', 'Battlefields', false, 5, (SELECT id FROM gods WHERE name = 'Ares')),
    ('Deimos', 'Formido', 'male', 'Terror, Dread', 'None', 'Battlefields', false, 5, (SELECT id FROM gods WHERE name = 'Ares')),
    ('Harmonia', 'Concordia', 'female', 'Harmony, Concord', 'Necklace', 'Thebes', false, 5, (SELECT id FROM gods WHERE name = 'Ares'));

-- Insert The Nine Muses (Daughters of Zeus and Mnemosyne)
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_id, second_parent_titan_id) VALUES
    ('Calliope', NULL, 'female', 'Epic Poetry', 'Writing Tablet, Stylus', 'Mount Helicon', false, 6, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Mnemosyne')),
    ('Clio', NULL, 'female', 'History', 'Scroll, Books', 'Mount Helicon', false, 6, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Mnemosyne')),
    ('Erato', NULL, 'female', 'Love Poetry, Lyric Poetry', 'Cithara, Lyre', 'Mount Helicon', false, 6, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Mnemosyne')),
    ('Euterpe', NULL, 'female', 'Music, Flutes', 'Aulos, Flute', 'Mount Helicon', false, 6, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Mnemosyne')),
    ('Melpomene', NULL, 'female', 'Tragedy', 'Tragic Mask, Sword', 'Mount Helicon', false, 6, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Mnemosyne')),
    ('Polyhymnia', NULL, 'female', 'Sacred Poetry, Hymns', 'Veil, Grapes', 'Mount Helicon', false, 6, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Mnemosyne')),
    ('Terpsichore', NULL, 'female', 'Dance, Choral Song', 'Lyre, Dancing', 'Mount Helicon', false, 6, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Mnemosyne')),
    ('Thalia', NULL, 'female', 'Comedy, Pastoral Poetry', 'Comic Mask, Ivy Wreath', 'Mount Helicon', false, 6, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Mnemosyne')),
    ('Urania', NULL, 'female', 'Astronomy, Astrology', 'Globe, Compass', 'Mount Helicon', false, 6, (SELECT id FROM gods WHERE name = 'Zeus'), (SELECT id FROM titans WHERE name = 'Mnemosyne'));

-- Insert gods from Titan lineages
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level, parent_titan_id) VALUES
    ('Iris', 'Arcus', 'female', 'Rainbow, Divine Messenger', 'Rainbow, Pitcher, Wings', 'Sky', false, 6, (SELECT id FROM titans WHERE name = 'Tethys')),
    ('Tyche', 'Fortuna', 'female', 'Fortune, Luck, Chance', 'Cornucopia, Wheel, Rudder', 'Earth', false, 5, (SELECT id FROM titans WHERE name = 'Oceanus'));

-- Insert children of Nyx (from primordials)
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level) VALUES
    ('Hypnos', 'Somnus', 'male', 'Sleep, Slumber', 'Poppy, Horn, Wings', 'Underworld', false, 6),
    ('Thanatos', 'Mors', 'male', 'Death, Mortality', 'Sword, Butterfly, Inverted Torch', 'Underworld', false, 7),
    ('Nemesis', 'Invidia', 'female', 'Revenge, Retribution, Balance', 'Scales, Sword, Whip, Wheel', 'Rhamnous', false, 6),
    ('Eris', 'Discordia', 'female', 'Discord, Strife, Chaos', 'Golden Apple', 'Battlefields', false, 6),
    ('Clotho', 'Nona', 'female', 'Fate - Spinner of Life Thread', 'Spindle', 'Cosmic Realm', false, 8),
    ('Lachesis', 'Decima', 'female', 'Fate - Allotter of Life', 'Measuring Rod', 'Cosmic Realm', false, 8),
    ('Atropos', 'Morta', 'female', 'Fate - Cutter of Life Thread', 'Shears', 'Cosmic Realm', false, 8),
    ('Morpheus', NULL, 'male', 'Dreams, Sleep Visions', 'Wings, Poppy', 'Dream World', false, 5),
    ('Charon', NULL, 'male', 'Ferrying Dead Souls', 'Oar, Boat', 'River Styx', false, 5),
    ('Aether', NULL, 'male', 'Light, Upper Air', 'Light', 'Upper Atmosphere', false, 6),
    ('Hemera', 'Dies', 'female', 'Day', 'Light', 'Sky', false, 6);

-- Insert Wind Gods (Anemoi)
INSERT INTO gods (name, roman_name, gender, domain, symbol, realm, is_olympian, power_level) VALUES
    ('Boreas', 'Aquilo', 'male', 'North Wind, Winter', 'Conch Shell, Wings', 'Thrace', false, 6),
    ('Notus', 'Auster', 'male', 'South Wind, Summer Storms', 'Water Jar', 'Ethiopia', false, 6),
    ('Zephyrus', 'Favonius', 'male', 'West Wind, Spring', 'Flowers', 'Elysium', false, 6),
    ('Eurus', 'Vulturnus', 'male', 'East Wind, Autumn', 'None', 'Near Helios Palace', false, 5);

-- Insert famous heroes
INSERT INTO heroes (name, gender, birth_place, patron_god_id, is_demigod, mortal_parent, divine_parent_id, fame_level, status, weapon, special_ability) VALUES
    ('Heracles', 'male', 'Thebes', (SELECT id FROM gods WHERE name = 'Athena'), true, 'Alcmene', (SELECT id FROM gods WHERE name = 'Zeus'), 10, 'deified', 'Club, Bow, Lion Pelt', 'Superhuman strength and endurance'),
    ('Perseus', 'male', 'Argos', (SELECT id FROM gods WHERE name = 'Athena'), true, 'Danae', (SELECT id FROM gods WHERE name = 'Zeus'), 9, 'deified', 'Harpe Sword, Kibisis', 'Flight with winged sandals, divine gifts'),
    ('Achilles', 'male', 'Phthia', (SELECT id FROM gods WHERE name = 'Athena'), true, 'Peleus', (SELECT id FROM gods WHERE name = 'Thetis'), 10, 'deceased', 'Spear, Shield of Achilles', 'Near invulnerability except heel'),
    ('Odysseus', 'male', 'Ithaca', (SELECT id FROM gods WHERE name = 'Athena'), false, NULL, NULL, 9, 'mortal', 'Bow of Odysseus', 'Cunning intelligence, strategy, eloquence'),
    ('Theseus', 'male', 'Troezen', (SELECT id FROM gods WHERE name = 'Athena'), true, 'Aethra', (SELECT id FROM gods WHERE name = 'Poseidon'), 8, 'deceased', 'Sword of Aegeus, Club', 'Great strength, courage, wisdom'),
    ('Jason', 'male', 'Iolcus', (SELECT id FROM gods WHERE name = 'Hera'), false, 'Aeson', NULL, 8, 'deceased', 'Sword', 'Leadership, navigation, favor of gods'),
    ('Bellerophon', 'male', 'Corinth', (SELECT id FROM gods WHERE name = 'Athena'), true, 'Eurynome', (SELECT id FROM gods WHERE name = 'Poseidon'), 7, 'deceased', 'Spear, Bridle of Pegasus', 'Taming of Pegasus, dragon-slaying'),
    ('Atalanta', 'female', 'Arcadia', (SELECT id FROM gods WHERE name = 'Artemis'), false, 'Iasus', NULL, 7, 'transformed', 'Bow, Javelin', 'Incredible speed, unmatched hunting skill'),
    ('Orpheus', 'male', 'Thrace', (SELECT id FROM gods WHERE name = 'Apollo'), true, 'Oeagrus', (SELECT id FROM gods WHERE name = 'Apollo'), 7, 'deceased', 'Lyre', 'Music that charms all living things and even death itself'),
    ('Cadmus', 'male', 'Phoenicia', (SELECT id FROM gods WHERE name = 'Athena'), false, 'Agenor', NULL, 6, 'transformed', 'Spear', 'Dragon slaying, city founding, alphabet bringer'),
    ('Ajax the Great', 'male', 'Salamis', (SELECT id FROM gods WHERE name = 'Athena'), false, 'Telamon', NULL, 8, 'deceased', 'Tower Shield, Spear', 'Immense size and strength, greatest warrior after Achilles'),
    ('Diomedes', 'male', 'Argos', (SELECT id FROM gods WHERE name = 'Athena'), false, 'Tydeus', NULL, 8, 'immortal', 'Spear, Sword', 'Wounded Ares and Aphrodite, fearless in battle'),
    ('Patroclus', 'male', 'Opus', (SELECT id FROM gods WHERE name = 'Athena'), false, 'Menoetius', NULL, 7, 'deceased', 'Armor of Achilles', 'Skilled warrior, beloved companion'),
    ('Hector', 'male', 'Troy', (SELECT id FROM gods WHERE name = 'Apollo'), false, 'Priam', NULL, 9, 'deceased', 'Spear, Sword', 'Greatest Trojan warrior, protector of Troy'),
    ('Aeneas', 'male', 'Dardania', (SELECT id FROM gods WHERE name = 'Aphrodite'), true, 'Anchises', (SELECT id FROM gods WHERE name = 'Aphrodite'), 8, 'deified', 'Spear, Shield', 'Piety, destined to found Rome'),
    ('Meleager', 'male', 'Calydon', (SELECT id FROM gods WHERE name = 'Artemis'), false, 'Oeneus', NULL, 7, 'deceased', 'Spear, Javelin', 'Great hunter, led Calydonian Boar hunt');

-- Insert legendary creatures and monsters
INSERT INTO creatures (name, type, description, homeland, threat_level, is_immortal, weakness, slain_by_hero_id) VALUES
    ('Medusa', 'Gorgon', 'Snake-haired woman whose gaze turns people to stone. The only mortal Gorgon sister.', 'Island near Sarpedon', 9, false, 'Reflection in polished shield, attacked while sleeping', 2),
    ('Minotaur', 'Beast', 'Half-man, half-bull creature born from Pasiphae and the Cretan Bull, imprisoned in the Labyrinth', 'Crete Labyrinth', 8, false, 'Sword through the heart, navigation of labyrinth', 5),
    ('Hydra', 'Serpent', 'Nine-headed water serpent that regrows two heads for each one cut off. Central head is immortal.', 'Lake Lerna', 10, true, 'Cauterize neck stumps with fire, bury immortal head', 1),
    ('Chimera', 'Beast', 'Fire-breathing monster with lion head, goat body, and serpent tail', 'Lycia', 9, false, 'Lead-tipped spear melted by its own breath', 7),
    ('Cerberus', 'Hound', 'Three-headed dog guarding the gates of the Underworld, serpent tail, mane of snakes', 'Underworld Gates', 9, true, 'Music, honey cakes, sleeping potion', NULL),
    ('Cyclops Polyphemus', 'Giant', 'One-eyed giant shepherd, son of Poseidon, who trapped Odysseus and his men', 'Sicily', 7, false, 'Blinded with heated wooden stake', 4),
    ('Nemean Lion', 'Beast', 'Enormous lion with impenetrable golden fur, claws sharper than swords', 'Nemea', 8, false, 'Strangled with bare hands, skinned with own claws', 1),
    ('Sphinx', 'Monster', 'Creature with human head, lion body, eagle wings who poses riddles. Killed those who failed.', 'Thebes', 7, false, 'Answering riddle correctly causes self-destruction', NULL),
    ('Scylla', 'Sea Monster', 'Six-headed sea monster living in narrow strait, each head has triple rows of teeth', 'Strait of Messina', 9, true, 'Cannot be killed, only avoided by sailing closer to her than Charybdis', NULL),
    ('Charybdis', 'Sea Monster', 'Massive whirlpool monster that swallows and regurgitates the sea three times daily', 'Strait of Messina', 9, true, 'Cannot be killed, only avoided by timing', NULL),
    ('Typhon', 'Giant', 'Most deadly creature in Greek mythology. Storm giant with hundred dragon heads, father of monsters.', 'Mount Etna', 10, true, 'Trapped under Mount Etna by Zeus', NULL),
    ('Echidna', 'Monster', 'Mother of monsters. Half-woman, half-serpent who birthed many legendary creatures.', 'Cave in Cilicia', 8, true, 'Killed by Argus Panoptes while sleeping', NULL),
    ('Python', 'Serpent', 'Gigantic earth-dragon born from the mud after the great flood, guarded Delphi', 'Mount Parnassus', 8, false, 'Slain by arrows of Apollo', NULL),
    ('Ladon', 'Dragon', 'Hundred-headed dragon that guarded the golden apples in the Garden of Hesperides', 'Garden of Hesperides', 8, true, 'Killed by Heracles or put to sleep', 1),
    ('Stymphalian Birds', 'Birds', 'Man-eating birds with bronze beaks and metallic feathers they could launch', 'Lake Stymphalia', 7, false, 'Frightened by Athena''s rattle, shot down', 1),
    ('Erymanthian Boar', 'Beast', 'Giant boar that ravaged Erymanthus and surrounding areas', 'Mount Erymanthus', 7, false, 'Driven into deep snow and captured', 1),
    ('Cretan Bull', 'Beast', 'Fire-breathing bull, father of the Minotaur, originally sent by Poseidon', 'Crete', 7, false, 'Captured bare-handed by Heracles', 1),
    ('Mares of Diomedes', 'Beasts', 'Four man-eating horses owned by the giant Diomedes of Thrace', 'Thrace', 8, false, 'Fed their master, became calm', 1),
    ('Colchian Dragon', 'Dragon', 'Sleepless dragon that guarded the Golden Fleece in Colchis', 'Colchis', 8, false, 'Put to sleep by Medea''s magic', NULL),
    ('Pegasus', 'Winged Horse', 'Divine immortal winged stallion born from Medusa''s blood when Perseus slew her', 'Born from Medusa', 5, true, 'Friendly when tamed with golden bridle', NULL),
    ('Cetus', 'Sea Monster', 'Sea monster sent by Poseidon to devour Andromeda, slain by Perseus', 'Ethiopia Coast', 8, false, 'Medusa''s gaze or Harpe sword', 2),
    ('Calydonian Boar', 'Beast', 'Monstrous boar sent by Artemis to ravage Calydon', 'Calydon', 8, false, 'First wounded by Atalanta, killed by Meleager', 8),
    ('Sow of Crommyon', 'Beast', 'Monstrous wild sow named Phaea that terrorized travelers', 'Crommyon', 6, false, 'Slain by Theseus', 5),
    ('Harpies', 'Bird-Women', 'Wind spirits with bird bodies and human faces that snatch food and people', 'Thrace', 6, true, 'Can be driven away by heroes with divine weapons', NULL),
    ('Giants (Alcyoneus)', 'Giant', 'Immortal giant who could not be killed in his homeland of Pallene', 'Pallene', 9, true, 'Dragged outside homeland and killed by Heracles', 1),
    ('Giants (Porphyrion)', 'Giant', 'King of the Giants, nearly overpowered Hera during Gigantomachy', 'Earth', 9, false, 'Struck by Zeus''s thunderbolt and Heracles'' arrows', 1);

-- Insert famous quests and labors (with proper timestamps - using 1000 BCE era as base)
INSERT INTO quests (name, hero_id, quest_type, difficulty, location, objective, reward, status, started_at, completed_at) VALUES
    ('Slay the Nemean Lion', 1, 'Labor of Heracles', 8, 'Nemea', 'Kill the invulnerable lion and bring back its pelt', 'Lion pelt as impenetrable armor', 'completed', '1200-01-01 00:00:00+00 BC', '1200-01-15 00:00:00+00 BC'),
    ('Slay the Lernaean Hydra', 1, 'Labor of Heracles', 10, 'Lake Lerna', 'Destroy the nine-headed serpent terrorizing Lerna', 'Poison for arrows from Hydra blood', 'completed', '1200-02-01 00:00:00+00 BC', '1200-02-20 00:00:00+00 BC'),
    ('Capture the Ceryneian Hind', 1, 'Labor of Heracles', 7, 'Arcadia', 'Capture sacred golden-horned deer of Artemis alive', 'Favor of Artemis', 'completed', '1200-03-01 00:00:00+00 BC', '1201-03-01 00:00:00+00 BC'),
    ('Capture the Erymanthian Boar', 1, 'Labor of Heracles', 7, 'Mount Erymanthus', 'Capture the giant boar alive', 'Captured boar shown to Eurystheus', 'completed', '1201-04-01 00:00:00+00 BC', '1201-05-01 00:00:00+00 BC'),
    ('Clean the Augean Stables', 1, 'Labor of Heracles', 6, 'Elis', 'Clean thirty years of filth from cattle stables in one day', 'One-tenth of cattle (disputed)', 'completed', '1201-06-01 00:00:00+00 BC', '1201-06-02 00:00:00+00 BC'),
    ('Slay the Stymphalian Birds', 1, 'Labor of Heracles', 7, 'Lake Stymphalia', 'Kill or drive away man-eating birds', 'Region freed from birds', 'completed', '1201-07-01 00:00:00+00 BC', '1201-07-15 00:00:00+00 BC'),
    ('Capture the Cretan Bull', 1, 'Labor of Heracles', 7, 'Crete', 'Capture the fire-breathing bull', 'Bull captured and later released', 'completed', '1201-08-01 00:00:00+00 BC', '1201-09-01 00:00:00+00 BC'),
    ('Steal the Mares of Diomedes', 1, 'Labor of Heracles', 8, 'Thrace', 'Steal the four man-eating horses', 'Horses tamed and released', 'completed', '1201-10-01 00:00:00+00 BC', '1201-11-01 00:00:00+00 BC'),
    ('Obtain the Belt of Hippolyta', 1, 'Labor of Heracles', 8, 'Themiscyra', 'Retrieve the war belt of the Amazon queen', 'Belt for Eurystheus''s daughter', 'completed', '1202-01-01 00:00:00+00 BC', '1202-03-01 00:00:00+00 BC'),
    ('Steal the Cattle of Geryon', 1, 'Labor of Heracles', 9, 'Erytheia', 'Steal the red cattle from the three-bodied giant', 'Cattle brought to Eurystheus', 'completed', '1202-04-01 00:00:00+00 BC', '1202-08-01 00:00:00+00 BC'),
    ('Steal the Apples of Hesperides', 1, 'Labor of Heracles', 9, 'Garden of Hesperides', 'Retrieve golden apples from the garden', 'Golden apples (returned by Athena)', 'completed', '1202-09-01 00:00:00+00 BC', '1203-01-01 00:00:00+00 BC'),
    ('Capture Cerberus', 1, 'Labor of Heracles', 10, 'Underworld', 'Capture the three-headed dog without weapons', 'Completion of labors, purification', 'completed', '1203-02-01 00:00:00+00 BC', '1203-04-01 00:00:00+00 BC'),
    ('Slay Medusa', 2, 'Quest', 9, 'Island of Sarpedon', 'Behead Medusa and bring back her head', 'Medusa''s head as weapon', 'completed', '1250-04-01 00:00:00+00 BC', '1250-06-01 00:00:00+00 BC'),
    ('Rescue Andromeda', 2, 'Quest', 7, 'Ethiopia', 'Save princess from sea monster Cetus', 'Marriage to Andromeda', 'completed', '1250-06-15 00:00:00+00 BC', '1250-07-15 00:00:00+00 BC'),
    ('Slay the Minotaur', 5, 'Quest', 8, 'Crete Labyrinth', 'Kill the Minotaur and escape the labyrinth', 'Freedom for Athens from tribute', 'completed', '1300-04-01 00:00:00+00 BC', '1300-04-20 00:00:00+00 BC'),
    ('Retrieve the Golden Fleece', 6, 'Quest', 10, 'Colchis', 'Steal the legendary Golden Fleece guarded by dragon', 'Golden Fleece and throne of Iolcus', 'completed', '1280-01-01 00:00:00+00 BC', '1280-08-30 00:00:00+00 BC'),
    ('Return to Ithaca', 4, 'Journey', 9, 'Mediterranean Sea', 'Navigate home after the fall of Troy', 'Reclaim kingdom, reunite with family', 'completed', '1188-06-01 00:00:00+00 BC', '1178-10-01 00:00:00+00 BC'),
    ('Defeat the Chimera', 7, 'Quest', 9, 'Lycia', 'Kill the fire-breathing Chimera terrorizing the land', 'King''s favor and princess''s hand', 'completed', '1290-05-01 00:00:00+00 BC', '1290-05-15 00:00:00+00 BC'),
    ('Journey to the Underworld', 9, 'Quest', 10, 'Underworld', 'Retrieve wife Eurydice from death', 'Failed - looked back at Eurydice too early', 'failed', '1270-01-01 00:00:00+00 BC', NULL),
    ('Calydonian Boar Hunt', 16, 'Quest', 8, 'Calydon', 'Hunt and kill the monstrous boar sent by Artemis', 'Boar''s hide and fame', 'completed', '1270-09-01 00:00:00+00 BC', '1270-09-10 00:00:00+00 BC'),
    ('The Trojan War', 3, 'War', 10, 'Troy', 'Fight for the Greeks and help conquer Troy', 'Glory and fame eternal', 'completed', '1194-01-01 00:00:00+00 BC', '1184-06-01 00:00:00+00 BC'),
    ('Avenge Patroclus', 3, 'Quest', 9, 'Troy', 'Kill Hector in revenge for slaying Patroclus', 'Vengeance achieved', 'completed', '1184-05-01 00:00:00+00 BC', '1184-05-15 00:00:00+00 BC');

-- Insert some in-progress quests for testing different statuses
INSERT INTO quests (name, hero_id, quest_type, difficulty, location, objective, reward, status, started_at, completed_at) VALUES
    ('Escape the Underworld', 5, 'Quest', 8, 'Underworld', 'Escape from Hades after failed attempt to kidnap Persephone', 'Freedom', 'in_progress', '1280-01-01 00:00:00+00 BC', NULL),
    ('Find the Founding of Rome', 15, 'Journey', 10, 'Mediterranean', 'Journey to Italy and establish a new civilization', 'Found Rome''s ancestor city', 'in_progress', '1184-07-01 00:00:00+00 BC', NULL),
    ('Win Hippodamia', NULL, 'Quest', 7, 'Pisa', 'Defeat King Oenomaus in chariot race', 'Marriage and kingdom', 'abandoned', '1300-01-01 00:00:00+00 BC', NULL);

-- Insert legendary artifacts
INSERT INTO artifacts (name, type, forged_by_god_id, current_owner_id, owner_type, power_description, material, is_cursed) VALUES
    ('Zeus''s Thunderbolt', 'Weapon', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Zeus'), 'god', 'Unlimited lightning strikes that can destroy anything, symbol of divine authority', 'Divine Bronze, Celestial Fire', false),
    ('Poseidon''s Trident', 'Weapon', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Poseidon'), 'god', 'Control over seas, earthquakes, creation of springs and horses', 'Divine Bronze', false),
    ('Hades'' Helm of Darkness', 'Helmet', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Hades'), 'god', 'Grants complete invisibility to the wearer, even from other gods', 'Divine Adamantine', false),
    ('Aegis', 'Shield', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Athena'), 'god', 'Shield or breastplate bearing Medusa''s head, causes terror in enemies', 'Divine Bronze, Goatskin', false),
    ('Hermes'' Winged Sandals (Talaria)', 'Footwear', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Hermes'), 'god', 'Grants ability to fly at incredible speeds', 'Divine Gold, Wings', false),
    ('Caduceus', 'Staff', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Hermes'), 'god', 'Herald''s staff with intertwined serpents, symbol of commerce and negotiation', 'Divine Gold', false),
    ('Golden Fleece', 'Artifact', NULL, 6, 'hero', 'Brings prosperity, healing, and legitimacy to kingship', 'Golden Wool of Chrysomallos', false),
    ('Pandora''s Box (Pithos)', 'Container', (SELECT id FROM gods WHERE name = 'Hephaestus'), NULL, 'lost', 'Originally contained all evils of the world, only Hope remains inside', 'Enchanted Clay', true),
    ('Bow of Heracles', 'Weapon', (SELECT id FROM gods WHERE name = 'Apollo'), 1, 'hero', 'Arrows never miss and are coated with Hydra poison', 'Divine Wood', false),
    ('Sword of Damocles', 'Weapon', NULL, NULL, 'lost', 'Represents imminent and ever-present peril faced by those in power', 'Steel', true),
    ('Ambrosia', 'Food', NULL, NULL, 'god', 'Food of the gods, grants immortality and heals all wounds', 'Divine Essence', false),
    ('Nectar', 'Drink', NULL, NULL, 'god', 'Drink of the gods, consumed with ambrosia for immortality', 'Divine Essence', false),
    ('Medusa''s Head (Gorgoneion)', 'Artifact', NULL, (SELECT id FROM gods WHERE name = 'Athena'), 'god', 'Turns any who look upon it to stone, mounted on Athena''s aegis', 'Petrified Gorgon', true),
    ('Thread of Ariadne', 'Tool', NULL, 5, 'hero', 'Magical thread that helped Theseus escape the labyrinth', 'Enchanted Thread', false),
    ('Orpheus''s Lyre', 'Instrument', (SELECT id FROM gods WHERE name = 'Apollo'), 9, 'hero', 'Music that charms all living things, beasts, trees, and even death itself', 'Divine Tortoise Shell, Strings', false),
    ('Talaria (Lent to Perseus)', 'Footwear', (SELECT id FROM gods WHERE name = 'Hephaestus'), 2, 'hero', 'Winged sandals granting flight, temporarily lent by Hermes', 'Divine Leather, Wings', false),
    ('Kibisis', 'Bag', NULL, 2, 'hero', 'Magical bag that safely contained Medusa''s head', 'Enchanted Leather', false),
    ('Harpe', 'Weapon', (SELECT id FROM gods WHERE name = 'Hephaestus'), 2, 'hero', 'Adamantine sword/sickle used to behead Medusa', 'Adamantine', false),
    ('Chariot of Helios', 'Vehicle', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Helios'), 'god', 'Sun chariot that drives across the sky each day, pulled by fire-breathing horses', 'Gold, Fire', false),
    ('Prometheus''s Torch', 'Artifact', NULL, (SELECT id FROM titans WHERE name = 'Prometheus'), 'titan', 'The sacred fire stolen from the gods and given to humanity', 'Divine Flame', false),
    ('Armor of Achilles', 'Armor', (SELECT id FROM gods WHERE name = 'Hephaestus'), 3, 'hero', 'Divine armor forged by Hephaestus at Thetis''s request, depicted the cosmos', 'Divine Bronze', false),
    ('Shield of Achilles', 'Shield', (SELECT id FROM gods WHERE name = 'Hephaestus'), 3, 'hero', 'Legendary shield depicting scenes of war and peace, cities and countryside', 'Divine Bronze', false),
    ('Bow of Apollo', 'Weapon', NULL, (SELECT id FROM gods WHERE name = 'Apollo'), 'god', 'Silver bow that brings plague or healing, arrows never miss', 'Divine Silver', false),
    ('Bow of Artemis', 'Weapon', NULL, (SELECT id FROM gods WHERE name = 'Artemis'), 'god', 'Golden bow for hunting, arrows grant swift painless death', 'Divine Gold', false),
    ('Cornucopia', 'Artifact', NULL, (SELECT id FROM gods WHERE name = 'Tyche'), 'god', 'Horn of plenty that produces endless food and drink', 'Divine Horn', false),
    ('Golden Apple of Discord', 'Artifact', NULL, (SELECT id FROM gods WHERE name = 'Eris'), 'god', 'Apple inscribed "For the Fairest" that caused the Trojan War', 'Divine Gold', true),
    ('Eros''s Bow and Arrows', 'Weapon', NULL, (SELECT id FROM gods WHERE name = 'Eros'), 'god', 'Gold arrows cause love, lead arrows cause hatred', 'Divine Gold and Lead', false),
    ('Necklace of Harmonia', 'Jewelry', (SELECT id FROM gods WHERE name = 'Hephaestus'), NULL, 'lost', 'Beautiful necklace that brings misfortune to all who possess it', 'Divine Gold, Gems', true),
    ('Spear of Achilles', 'Weapon', NULL, 3, 'hero', 'Pelian spear given by Chiron, only Achilles could wield it', 'Ash Wood from Mount Pelion', false);

-- Insert Titan artifacts
INSERT INTO artifacts (name, type, forged_by_titan_id, current_owner_id, owner_type, power_description, material, is_cursed) VALUES
    ('Scythe of Cronus', 'Weapon', NULL, (SELECT id FROM titans WHERE name = 'Cronus'), 'titan', 'Adamantine weapon used to castrate Uranus, can cut through anything divine', 'Adamantine', true),
    ('Chains of Prometheus', 'Binding', NULL, (SELECT id FROM titans WHERE name = 'Prometheus'), 'titan', 'Unbreakable chains that bound Prometheus to the Caucasus', 'Divine Adamantine', true);

-- Insert key relationships
INSERT INTO relationships (entity1_id, entity1_type, entity2_id, entity2_type, relationship_type, description) VALUES
    ((SELECT id FROM gods WHERE name = 'Zeus'), 'god', (SELECT id FROM gods WHERE name = 'Hera'), 'god', 'spouse', 'King and Queen of the Gods, frequently at odds over Zeus''s affairs'),
    ((SELECT id FROM gods WHERE name = 'Hades'), 'god', (SELECT id FROM gods WHERE name = 'Persephone'), 'god', 'spouse', 'Lord and Lady of the Underworld, spend half the year apart'),
    ((SELECT id FROM gods WHERE name = 'Ares'), 'god', (SELECT id FROM gods WHERE name = 'Aphrodite'), 'god', 'lover', 'Famous divine affair, parents of Eros, Phobos, Deimos, and Harmonia'),
    ((SELECT id FROM gods WHERE name = 'Athena'), 'god', (SELECT id FROM gods WHERE name = 'Poseidon'), 'god', 'rival', 'Competed for patronage of Athens, ongoing rivalry'),
    ((SELECT id FROM gods WHERE name = 'Hera'), 'god', (SELECT id FROM heroes WHERE name = 'Heracles'), 'hero', 'enemy', 'Hera persecuted Heracles throughout his life due to his being Zeus''s illegitimate son'),
    ((SELECT id FROM gods WHERE name = 'Athena'), 'god', (SELECT id FROM heroes WHERE name = 'Odysseus'), 'hero', 'ally', 'Athena consistently aided Odysseus throughout his journeys'),
    ((SELECT id FROM gods WHERE name = 'Poseidon'), 'god', (SELECT id FROM heroes WHERE name = 'Odysseus'), 'hero', 'enemy', 'Poseidon cursed Odysseus for blinding his son Polyphemus'),
    ((SELECT id FROM gods WHERE name = 'Apollo'), 'god', (SELECT id FROM gods WHERE name = 'Artemis'), 'god', 'ally', 'Twin siblings with close bond'),
    ((SELECT id FROM gods WHERE name = 'Hephaestus'), 'god', (SELECT id FROM gods WHERE name = 'Aphrodite'), 'god', 'spouse', 'Arranged marriage, Aphrodite was unfaithful'),
    ((SELECT id FROM titans WHERE name = 'Prometheus'), 'titan', (SELECT id FROM gods WHERE name = 'Zeus'), 'god', 'enemy', 'Prometheus stole fire for humanity, punished eternally'),
    ((SELECT id FROM heroes WHERE name = 'Achilles'), 'hero', (SELECT id FROM heroes WHERE name = 'Patroclus'), 'hero', 'ally', 'Beloved companion, Patroclus''s death drove Achilles to rejoin the war'),
    ((SELECT id FROM heroes WHERE name = 'Achilles'), 'hero', (SELECT id FROM heroes WHERE name = 'Hector'), 'hero', 'rival', 'Greatest warriors of Greece and Troy, Achilles slew Hector');

-- Create indexes for better performance
CREATE INDEX idx_primordials_name ON primordials(name);
CREATE INDEX idx_primordials_gender ON primordials(gender);
CREATE INDEX idx_titans_parent_primordial ON titans(parent_primordial_id);
CREATE INDEX idx_titans_parent_titan ON titans(parent_titan_id);
CREATE INDEX idx_titans_name ON titans(name);
CREATE INDEX idx_titans_gender ON titans(gender);
CREATE INDEX idx_gods_domain ON gods(domain);
CREATE INDEX idx_gods_is_olympian ON gods(is_olympian);
CREATE INDEX idx_gods_parent ON gods(parent_id);
CREATE INDEX idx_gods_parent_titan ON gods(parent_titan_id);
CREATE INDEX idx_gods_gender ON gods(gender);
CREATE INDEX idx_gods_spouse ON gods(spouse_id);
CREATE INDEX idx_heroes_patron_god ON heroes(patron_god_id);
CREATE INDEX idx_heroes_is_demigod ON heroes(is_demigod);
CREATE INDEX idx_heroes_gender ON heroes(gender);
CREATE INDEX idx_heroes_status ON heroes(status);
CREATE INDEX idx_creatures_type ON creatures(type);
CREATE INDEX idx_creatures_slain_by ON creatures(slain_by_hero_id);
CREATE INDEX idx_creatures_threat_level ON creatures(threat_level);
CREATE INDEX idx_quests_hero ON quests(hero_id);
CREATE INDEX idx_quests_status ON quests(status);
CREATE INDEX idx_quests_type ON quests(quest_type);
CREATE INDEX idx_artifacts_owner ON artifacts(current_owner_id, owner_type);
CREATE INDEX idx_artifacts_cursed ON artifacts(is_cursed);
CREATE INDEX idx_relationships_entity1 ON relationships(entity1_id, entity1_type);
CREATE INDEX idx_relationships_entity2 ON relationships(entity2_id, entity2_type);
CREATE INDEX idx_relationships_type ON relationships(relationship_type);

-- Create a view for hero achievements
CREATE VIEW hero_achievements AS
SELECT
    h.name as hero_name,
    h.gender,
    h.birth_place,
    g.name as patron_god,
    h.is_demigod,
    dp.name as divine_parent,
    h.fame_level,
    h.status,
    h.weapon,
    h.special_ability,
    COUNT(DISTINCT q.id) FILTER (WHERE q.status = 'completed') as quests_completed,
    COUNT(DISTINCT q.id) FILTER (WHERE q.status = 'failed') as quests_failed,
    COUNT(DISTINCT q.id) FILTER (WHERE q.status = 'in_progress') as quests_in_progress,
    COUNT(DISTINCT c.id) as monsters_slain,
    STRING_AGG(DISTINCT c.name, ', ') as defeated_creatures
FROM heroes h
LEFT JOIN gods g ON h.patron_god_id = g.id
LEFT JOIN gods dp ON h.divine_parent_id = dp.id
LEFT JOIN quests q ON h.id = q.hero_id
LEFT JOIN creatures c ON h.id = c.slain_by_hero_id
GROUP BY h.id, h.name, h.gender, h.birth_place, g.name, h.is_demigod, dp.name, h.fame_level, h.status, h.weapon, h.special_ability;

-- Create a view for complete divine lineage
CREATE VIEW divine_lineage AS
-- Gods with god parents
SELECT
    child.name as entity_name,
    child.gender,
    'God' as entity_type,
    child.domain,
    parent.name as parent_name,
    'God' as parent_type,
    parent.domain as parent_domain,
    child.is_olympian,
    child.birth_method
FROM gods child
LEFT JOIN gods parent ON child.parent_id = parent.id
WHERE child.parent_id IS NOT NULL
UNION ALL
-- Gods with titan parents
SELECT
    child.name as entity_name,
    child.gender,
    'God' as entity_type,
    child.domain,
    parent.name as parent_name,
    'Titan' as parent_type,
    parent.domain as parent_domain,
    child.is_olympian,
    child.birth_method
FROM gods child
LEFT JOIN titans parent ON child.parent_titan_id = parent.id
WHERE child.parent_titan_id IS NOT NULL
UNION ALL
-- Titans with primordial parents
SELECT
    child.name as entity_name,
    child.gender,
    'Titan' as entity_type,
    child.domain,
    parent.name as parent_name,
    'Primordial' as parent_type,
    parent.domain as parent_domain,
    false as is_olympian,
    NULL as birth_method
FROM titans child
LEFT JOIN primordials parent ON child.parent_primordial_id = parent.id
WHERE child.parent_primordial_id IS NOT NULL
UNION ALL
-- Titans with titan parents (second generation)
SELECT
    child.name as entity_name,
    child.gender,
    'Titan' as entity_type,
    child.domain,
    parent.name as parent_name,
    'Titan' as parent_type,
    parent.domain as parent_domain,
    false as is_olympian,
    NULL as birth_method
FROM titans child
LEFT JOIN titans parent ON child.parent_titan_id = parent.id
WHERE child.parent_titan_id IS NOT NULL;

-- Create a view for artifact ownership
CREATE VIEW artifact_registry AS
SELECT
    a.name as artifact_name,
    a.type,
    a.is_cursed,
    COALESCE(forger_god.name, forger_titan.name, 'Unknown Origin') as forged_by,
    CASE
        WHEN a.owner_type = 'god' THEN g.name
        WHEN a.owner_type = 'hero' THEN h.name
        WHEN a.owner_type = 'creature' THEN c.name
        WHEN a.owner_type = 'titan' THEN t.name
        ELSE 'Unknown/Lost'
    END as current_owner,
    a.owner_type,
    a.power_description,
    a.material
FROM artifacts a
LEFT JOIN gods forger_god ON a.forged_by_god_id = forger_god.id
LEFT JOIN titans forger_titan ON a.forged_by_titan_id = forger_titan.id
LEFT JOIN gods g ON a.current_owner_id = g.id AND a.owner_type = 'god'
LEFT JOIN heroes h ON a.current_owner_id = h.id AND a.owner_type = 'hero'
LEFT JOIN creatures c ON a.current_owner_id = c.id AND a.owner_type = 'creature'
LEFT JOIN titans t ON a.current_owner_id = t.id AND a.owner_type = 'titan';

-- Create a view for Olympian family tree
CREATE VIEW olympian_family AS
SELECT
    g.name,
    g.roman_name,
    g.gender,
    g.domain,
    g.symbol,
    g.power_level,
    COALESCE(p1.name, pt1.name) as parent_1,
    COALESCE(p2.name, pt2.name) as parent_2,
    s.name as spouse,
    g.birth_method
FROM gods g
LEFT JOIN gods p1 ON g.parent_id = p1.id
LEFT JOIN titans pt1 ON g.parent_titan_id = pt1.id
LEFT JOIN gods p2 ON g.second_parent_id = p2.id
LEFT JOIN titans pt2 ON g.second_parent_titan_id = pt2.id
LEFT JOIN gods s ON g.spouse_id = s.id
WHERE g.is_olympian = true
ORDER BY g.power_level DESC;

-- Create a view for creature threat assessment
CREATE VIEW creature_threat_assessment AS
SELECT
    c.name,
    c.type,
    c.threat_level,
    c.is_immortal,
    c.homeland,
    c.weakness,
    h.name as slain_by,
    c.description
FROM creatures c
LEFT JOIN heroes h ON c.slain_by_hero_id = h.id
ORDER BY c.threat_level DESC, c.name;

-- Create a summary view of all entities
CREATE VIEW mythology_summary AS
SELECT 'Primordials' as category, COUNT(*) as count FROM primordials
UNION ALL
SELECT 'Titans' as category, COUNT(*) as count FROM titans
UNION ALL
SELECT 'Gods' as category, COUNT(*) as count FROM gods
UNION ALL
SELECT 'Olympians' as category, COUNT(*) as count FROM gods WHERE is_olympian = true
UNION ALL
SELECT 'Heroes' as category, COUNT(*) as count FROM heroes
UNION ALL
SELECT 'Demigods' as category, COUNT(*) as count FROM heroes WHERE is_demigod = true
UNION ALL
SELECT 'Creatures' as category, COUNT(*) as count FROM creatures
UNION ALL
SELECT 'Immortal Creatures' as category, COUNT(*) as count FROM creatures WHERE is_immortal = true
UNION ALL
SELECT 'Quests' as category, COUNT(*) as count FROM quests
UNION ALL
SELECT 'Completed Quests' as category, COUNT(*) as count FROM quests WHERE status = 'completed'
UNION ALL
SELECT 'Artifacts' as category, COUNT(*) as count FROM artifacts
UNION ALL
SELECT 'Cursed Artifacts' as category, COUNT(*) as count FROM artifacts WHERE is_cursed = true
UNION ALL
SELECT 'Relationships' as category, COUNT(*) as count FROM relationships;

-- Create a view for divine relationships network
CREATE VIEW divine_relationships AS
SELECT
    CASE
        WHEN r.entity1_type = 'god' THEN g1.name
        WHEN r.entity1_type = 'titan' THEN t1.name
        WHEN r.entity1_type = 'hero' THEN h1.name
        WHEN r.entity1_type = 'primordial' THEN p1.name
    END as entity1_name,
    r.entity1_type,
    r.relationship_type,
    CASE
        WHEN r.entity2_type = 'god' THEN g2.name
        WHEN r.entity2_type = 'titan' THEN t2.name
        WHEN r.entity2_type = 'hero' THEN h2.name
        WHEN r.entity2_type = 'primordial' THEN p2.name
    END as entity2_name,
    r.entity2_type,
    r.description
FROM relationships r
LEFT JOIN gods g1 ON r.entity1_id = g1.id AND r.entity1_type = 'god'
LEFT JOIN titans t1 ON r.entity1_id = t1.id AND r.entity1_type = 'titan'
LEFT JOIN heroes h1 ON r.entity1_id = h1.id AND r.entity1_type = 'hero'
LEFT JOIN primordials p1 ON r.entity1_id = p1.id AND r.entity1_type = 'primordial'
LEFT JOIN gods g2 ON r.entity2_id = g2.id AND r.entity2_type = 'god'
LEFT JOIN titans t2 ON r.entity2_id = t2.id AND r.entity2_type = 'titan'
LEFT JOIN heroes h2 ON r.entity2_id = h2.id AND r.entity2_type = 'hero'
LEFT JOIN primordials p2 ON r.entity2_id = p2.id AND r.entity2_type = 'primordial';

-- =====================================================
-- Test Roles for Videre Roles UI
-- =====================================================

-- Create a read-only role for oracle seers (can only SELECT)
CREATE ROLE oracle_reader WITH
    LOGIN
    PASSWORD 'delphi123'
    NOSUPERUSER
    NOCREATEDB
    NOCREATEROLE
    CONNECTION LIMIT 10;

-- Grant read-only access to all tables
GRANT CONNECT ON DATABASE videre_test TO oracle_reader;
GRANT USAGE ON SCHEMA public TO oracle_reader;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO oracle_reader;

-- Create a temple administrator role (can read/write but not admin)
CREATE ROLE temple_admin WITH
    LOGIN
    PASSWORD 'olympus456'
    NOSUPERUSER
    NOCREATEDB
    NOCREATEROLE
    CONNECTION LIMIT 50
    VALID UNTIL '2026-12-31';

-- Grant read/write access
GRANT CONNECT ON DATABASE videre_test TO temple_admin;
GRANT USAGE ON SCHEMA public TO temple_admin;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO temple_admin;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO temple_admin;

-- Create a DBA role for Mount Olympus (full admin powers)
CREATE ROLE olympus_dba WITH
    LOGIN
    PASSWORD 'zeus789'
    SUPERUSER
    CREATEDB
    CREATEROLE
    CONNECTION LIMIT -1;

-- Create a group role for readonly access (no login, used for inheritance)
CREATE ROLE readonly_group WITH
    NOLOGIN
    NOSUPERUSER
    NOCREATEDB
    NOCREATEROLE;

GRANT USAGE ON SCHEMA public TO readonly_group;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly_group;

-- Make oracle_reader a member of readonly_group (inherits permissions)
GRANT readonly_group TO oracle_reader;
