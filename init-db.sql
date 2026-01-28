-- Videre Test Database Initialization Script
-- Greek Mythology themed database for testing

-- Primordial Deities table (First Generation)
CREATE TABLE primordials (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
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
    domain VARCHAR(100) NOT NULL,
    symbol VARCHAR(100),
    realm VARCHAR(50),
    parent_id INTEGER REFERENCES primordials(id),
    power_level INTEGER CHECK (power_level >= 1 AND power_level <= 10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Gods and Goddesses table (Olympians and their descendants)
CREATE TABLE gods (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    roman_name VARCHAR(100),
    domain VARCHAR(100) NOT NULL,
    symbol VARCHAR(100),
    realm VARCHAR(50),
    parent_id INTEGER REFERENCES gods(id),
    parent_titan_id INTEGER REFERENCES titans(id),
    is_olympian BOOLEAN DEFAULT false,
    power_level INTEGER CHECK (power_level >= 1 AND power_level <= 10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Heroes table
CREATE TABLE heroes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
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
    status VARCHAR(50) DEFAULT 'in_progress',
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

-- Insert Primordial Deities (Born from Chaos)
INSERT INTO primordials (name, domain, symbol, realm, power_level) VALUES
    ('Chaos', 'The Void, Nothingness', 'Abyss', 'The Void', 10),
    ('Gaia', 'Earth, Mother of All', 'Earth, Fertile Soil', 'Earth', 10),
    ('Uranus', 'Sky, Heavens', 'Starry Sky', 'Sky', 10),
    ('Nyx', 'Night, Darkness', 'Stars, Black Cloak', 'Tartarus', 9),
    ('Erebus', 'Darkness, Shadow', 'Mist, Shadows', 'Underworld', 9),
    ('Tartarus', 'The Abyss, Deepest Pit', 'Prison Chains', 'Deepest Underworld', 9),
    ('Eros (Primordial)', 'Primordial Love, Procreation', 'None', 'Everywhere', 8);

-- Insert the Titans (Children of Gaia and Uranus)
INSERT INTO titans (name, roman_name, domain, symbol, realm, power_level, parent_id) VALUES
    ('Cronus', 'Saturn', 'Time, King of Titans', 'Sickle, Scythe', 'Mount Othrys', 10, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Rhea', 'Ops', 'Fertility, Motherhood', 'Lion, Turret Crown', 'Mount Othrys', 9, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Oceanus', NULL, 'Ocean, World River', 'Serpent, Fish', 'Ocean Stream', 9, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Tethys', NULL, 'Fresh Water, Nursing', 'Water Jug', 'Ocean', 8, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Hyperion', NULL, 'Light, Watchfulness', 'Sun', 'Sky', 8, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Theia', NULL, 'Sight, Brilliance', 'Shining Light', 'Sky', 8, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Coeus', NULL, 'Intelligence, Inquiry', 'Stars', 'North Pillar', 7, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Phoebe', NULL, 'Prophecy, Intellect', 'Moon', 'Delphi', 7, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Themis', NULL, 'Divine Law, Order', 'Scales, Sword', 'Mount Olympus', 8, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Mnemosyne', NULL, 'Memory, Remembrance', 'Lamp', 'Pieria', 7, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Iapetus', NULL, 'Mortality, Craftiness', 'Spear', 'West Pillar', 7, (SELECT id FROM primordials WHERE name = 'Gaia')),
    ('Crius', NULL, 'Constellations, Heavenly Bodies', 'Ram', 'South Pillar', 7, (SELECT id FROM primordials WHERE name = 'Gaia'));

-- Insert Children of Titans (Pre-Olympian Deities)
INSERT INTO gods (name, roman_name, domain, symbol, realm, is_olympian, power_level, parent_titan_id) VALUES
    ('Helios', 'Sol', 'The Sun, Light of Day', 'Sun Chariot, Crown of Rays', 'Sky', false, 8, (SELECT id FROM titans WHERE name = 'Hyperion')),
    ('Selene', 'Luna', 'The Moon, Night Light', 'Moon Crescent, Torch', 'Sky', false, 7, (SELECT id FROM titans WHERE name = 'Hyperion')),
    ('Eos', 'Aurora', 'Dawn, Morning', 'Roses, Saffron Robe', 'Sky', false, 7, (SELECT id FROM titans WHERE name = 'Hyperion')),
    ('Prometheus', NULL, 'Forethought, Fire Giver', 'Torch, Fennel Staff', 'Earth', false, 7, (SELECT id FROM titans WHERE name = 'Iapetus')),
    ('Atlas', NULL, 'Endurance, Astronomy', 'Celestial Sphere', 'Edge of World', false, 8, (SELECT id FROM titans WHERE name = 'Iapetus')),
    ('Epimetheus', NULL, 'Afterthought, Excuses', 'None', 'Earth', false, 5, (SELECT id FROM titans WHERE name = 'Iapetus')),
    ('Hecate', 'Trivia', 'Magic, Crossroads, Necromancy', 'Torch, Keys, Dagger', 'Crossroads', false, 8, (SELECT id FROM titans WHERE name = 'Coeus'));

-- Insert the Olympian Gods (Children of Cronus and Rhea)
INSERT INTO gods (name, roman_name, domain, symbol, realm, is_olympian, power_level, parent_titan_id) VALUES
    ('Zeus', 'Jupiter', 'King of Gods, Sky and Thunder', 'Lightning Bolt, Eagle', 'Mount Olympus', true, 10, (SELECT id FROM titans WHERE name = 'Cronus')),
    ('Hera', 'Juno', 'Queen of Gods, Marriage', 'Peacock, Cow', 'Mount Olympus', true, 9, (SELECT id FROM titans WHERE name = 'Cronus')),
    ('Poseidon', 'Neptune', 'Sea, Earthquakes, Horses', 'Trident, Horse', 'Underwater Palace', true, 10, (SELECT id FROM titans WHERE name = 'Cronus')),
    ('Demeter', 'Ceres', 'Agriculture, Harvest', 'Wheat, Cornucopia', 'Fields', true, 8, (SELECT id FROM titans WHERE name = 'Cronus')),
    ('Hades', 'Pluto', 'Underworld, Dead, Wealth', 'Helm of Darkness, Cerberus', 'Underworld', false, 10, (SELECT id FROM titans WHERE name = 'Cronus')),
    ('Hestia', 'Vesta', 'Hearth, Home, Family', 'Flame, Kettle', 'Mount Olympus', false, 7, (SELECT id FROM titans WHERE name = 'Cronus'));

-- Insert Children of Zeus and other Olympians
INSERT INTO gods (name, roman_name, domain, symbol, realm, is_olympian, power_level, parent_id) VALUES
    ('Athena', 'Minerva', 'Wisdom, Warfare, Crafts', 'Owl, Olive Tree', 'Mount Olympus', true, 9, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Apollo', 'Apollo', 'Sun, Music, Prophecy, Healing', 'Lyre, Sun Chariot', 'Mount Olympus', true, 9, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Artemis', 'Diana', 'Moon, Hunt, Wilderness', 'Bow and Arrow, Deer', 'Forest', true, 9, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Ares', 'Mars', 'War, Violence', 'Spear, Vulture', 'Mount Olympus', true, 8, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Hephaestus', 'Vulcan', 'Fire, Forges, Metalworking', 'Hammer, Anvil', 'Volcano', true, 8, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Hermes', 'Mercury', 'Messengers, Trade, Thieves', 'Winged Sandals, Caduceus', 'Mount Olympus', true, 8, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Dionysus', 'Bacchus', 'Wine, Festivity, Theatre', 'Grapes, Thyrsus', 'Vineyards', true, 7, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Persephone', 'Proserpina', 'Spring, Queen of Underworld', 'Pomegranate, Flowers', 'Underworld', false, 7, (SELECT id FROM gods WHERE name = 'Demeter'));

-- Insert Aphrodite (special birth - from Uranus)
INSERT INTO gods (name, roman_name, domain, symbol, realm, is_olympian, power_level) VALUES
    ('Aphrodite', 'Venus', 'Love, Beauty', 'Dove, Rose', 'Mount Olympus', true, 8);

-- Insert Minor Gods and Personifications
INSERT INTO gods (name, roman_name, domain, symbol, realm, is_olympian, power_level, parent_id) VALUES
    ('Eros (Younger)', 'Cupid', 'Love, Desire, Attraction', 'Bow and Arrow, Wings', 'Mount Olympus', false, 6, (SELECT id FROM gods WHERE name = 'Aphrodite')),
    ('Nike', 'Victoria', 'Victory, Success', 'Wings, Laurel Wreath', 'Mount Olympus', false, 6, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('The Muses', 'Musae', 'Arts, Sciences, Inspiration', 'Various Instruments', 'Mount Helicon', false, 7, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Hebe', 'Juventas', 'Youth, Cupbearer', 'Cup, Fountain', 'Mount Olympus', false, 5, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Eileithyia', 'Lucina', 'Childbirth, Labor', 'Torch', 'Mount Olympus', false, 6, (SELECT id FROM gods WHERE name = 'Zeus')),
    ('Pan', 'Faunus', 'Nature, Shepherds, Flocks', 'Pan Flute, Shepherd Staff', 'Arcadia', false, 6, (SELECT id FROM gods WHERE name = 'Hermes'));

-- Insert gods from other lineages
INSERT INTO gods (name, roman_name, domain, symbol, realm, is_olympian, power_level, parent_titan_id) VALUES
    ('Iris', 'Arcus', 'Rainbow, Divine Messenger', 'Rainbow, Pitcher', 'Sky', false, 6, (SELECT id FROM titans WHERE name = 'Tethys')),
    ('Tyche', 'Fortuna', 'Fortune, Luck', 'Cornucopia, Wheel', 'Earth', false, 5, (SELECT id FROM titans WHERE name = 'Oceanus'));

-- Insert children of Nyx (from primordials)
INSERT INTO gods (name, roman_name, domain, symbol, realm, is_olympian, power_level) VALUES
    ('Hypnos', 'Somnus', 'Sleep, Slumber', 'Poppy, Horn', 'Underworld', false, 6),
    ('Thanatos', 'Mors', 'Death, Mortality', 'Sword, Butterfly', 'Underworld', false, 7),
    ('Nemesis', NULL, 'Revenge, Retribution', 'Scales, Sword, Whip', 'Mount Olympus', false, 6),
    ('Eris', 'Discordia', 'Discord, Strife, Chaos', 'Golden Apple', 'Battlefields', false, 6),
    ('The Fates (Moirai)', 'Parcae', 'Destiny, Life Thread', 'Thread, Shears', 'Cosmic Realm', false, 9),
    ('Morpheus', NULL, 'Dreams, Sleep Visions', 'Wings, Poppy', 'Dream World', false, 5);

-- Insert famous heroes
INSERT INTO heroes (name, birth_place, patron_god_id, is_demigod, divine_parent_id, fame_level, status, weapon, special_ability) VALUES
    ('Heracles', 'Thebes', (SELECT id FROM gods WHERE name = 'Athena'), true, (SELECT id FROM gods WHERE name = 'Zeus'), 10, 'deified', 'Club and Bow', 'Superhuman strength and endurance'),
    ('Perseus', 'Argos', (SELECT id FROM gods WHERE name = 'Athena'), true, (SELECT id FROM gods WHERE name = 'Zeus'), 9, 'deified', 'Harpe Sword', 'Flight with winged sandals'),
    ('Achilles', 'Phthia', (SELECT id FROM gods WHERE name = 'Athena'), true, NULL, 10, 'deceased', 'Spear and Shield', 'Near invulnerability except heel'),
    ('Odysseus', 'Ithaca', (SELECT id FROM gods WHERE name = 'Athena'), false, NULL, 9, 'mortal', 'Bow', 'Cunning intelligence and strategy'),
    ('Theseus', 'Athens', (SELECT id FROM gods WHERE name = 'Poseidon'), true, (SELECT id FROM gods WHERE name = 'Poseidon'), 8, 'deceased', 'Sword of Aegeus', 'Great strength and courage'),
    ('Jason', 'Iolcus', (SELECT id FROM gods WHERE name = 'Hera'), false, NULL, 8, 'mortal', 'Sword', 'Leadership and navigation'),
    ('Bellerophon', 'Corinth', (SELECT id FROM gods WHERE name = 'Athena'), false, NULL, 7, 'mortal', 'Spear', 'Taming of Pegasus'),
    ('Atalanta', 'Arcadia', (SELECT id FROM gods WHERE name = 'Artemis'), false, NULL, 7, 'mortal', 'Bow and Javelin', 'Incredible speed and hunting skill'),
    ('Orpheus', 'Thrace', (SELECT id FROM gods WHERE name = 'Apollo'), true, (SELECT id FROM gods WHERE name = 'Apollo'), 7, 'deceased', 'Lyre', 'Music that charms all living things'),
    ('Cadmus', 'Phoenicia', (SELECT id FROM gods WHERE name = 'Athena'), false, NULL, 6, 'transformed', 'Spear', 'Dragon slaying and city founding');

-- Insert legendary creatures and monsters
INSERT INTO creatures (name, type, description, homeland, threat_level, is_immortal, weakness, slain_by_hero_id) VALUES
    ('Medusa', 'Gorgon', 'Snake-haired woman whose gaze turns people to stone', 'Cave near Sarpedon', 9, false, 'Reflection in polished shield', 2),
    ('Minotaur', 'Beast', 'Half-man, half-bull creature trapped in a labyrinth', 'Crete Labyrinth', 8, false, 'Sword through the heart', 5),
    ('Hydra', 'Serpent', 'Nine-headed water serpent that regrows two heads for each one cut', 'Lake Lerna', 10, true, 'Cauterize neck stumps with fire', 1),
    ('Chimera', 'Beast', 'Fire-breathing monster with lion head, goat body, serpent tail', 'Lycia', 9, false, 'Spear with lead tip melted by its own breath', 7),
    ('Cerberus', 'Hound', 'Three-headed dog guarding the gates of the Underworld', 'Underworld Gates', 9, true, 'Music and honey cakes', NULL),
    ('Cyclops Polyphemus', 'Giant', 'One-eyed giant shepherd who trapped Odysseus', 'Sicily', 7, false, 'Blinded with wooden stake', 4),
    ('Nemean Lion', 'Beast', 'Enormous lion with impenetrable golden fur', 'Nemea', 8, false, 'Strangled with bare hands', 1),
    ('Sphinx', 'Monster', 'Creature with human head, lion body, bird wings who poses riddles', 'Thebes', 7, false, 'Answering riddle correctly', NULL),
    ('Scylla', 'Sea Monster', 'Six-headed sea monster living in narrow strait', 'Strait of Messina', 9, true, 'Cannot be killed, only avoided', NULL),
    ('Charybdis', 'Sea Monster', 'Massive whirlpool monster opposite Scylla', 'Strait of Messina', 9, true, 'Cannot be killed, only avoided', NULL),
    ('Harpies', 'Bird-Women', 'Wind spirits that snatch food and people', 'Thrace', 6, true, 'Can be driven away by heroes', NULL),
    ('Python', 'Serpent', 'Gigantic earth-dragon guarding Delphi', 'Mount Parnassus', 8, false, 'Arrows', NULL),
    ('Pegasus', 'Winged Horse', 'Divine immortal winged stallion', 'Born from Medusa', 5, true, 'Friendly when tamed', NULL);

-- Insert famous quests and labors
INSERT INTO quests (name, hero_id, quest_type, difficulty, location, objective, reward, status, completed_at) VALUES
    ('Slay the Nemean Lion', 1, 'Labor of Heracles', 8, 'Nemea', 'Kill the invulnerable lion and bring back its pelt', 'Lion pelt as armor', 'completed', '1200-01-15 BC'),
    ('Slay the Lernaean Hydra', 1, 'Labor of Heracles', 10, 'Lake Lerna', 'Destroy the nine-headed serpent', 'Poison for arrows', 'completed', '1200-02-20 BC'),
    ('Capture the Golden Hind', 1, 'Labor of Heracles', 7, 'Arcadia', 'Capture sacred deer of Artemis alive', 'Favor of Artemis', 'completed', '1200-03-10 BC'),
    ('Slay Medusa', 2, 'Quest', 9, 'Island of Sarpedon', 'Behead Medusa and bring back her head', 'Medusa''s head as weapon', 'completed', '1250-06-01 BC'),
    ('Rescue Andromeda', 2, 'Quest', 7, 'Ethiopia', 'Save princess from sea monster Cetus', 'Marriage to Andromeda', 'completed', '1250-07-15 BC'),
    ('Slay the Minotaur', 5, 'Quest', 8, 'Crete Labyrinth', 'Kill the Minotaur and escape the labyrinth', 'Freedom for Athens from tribute', 'completed', '1300-04-20 BC'),
    ('Retrieve the Golden Fleece', 6, 'Quest', 10, 'Colchis', 'Steal the legendary Golden Fleece guarded by dragon', 'Golden Fleece and throne of Iolcus', 'completed', '1280-08-30 BC'),
    ('Return to Ithaca', 4, 'Journey', 9, 'Mediterranean Sea', 'Navigate home after Trojan War', 'Reclaim kingdom and family', 'completed', '1178-10-01 BC'),
    ('Defeat the Chimera', 7, 'Quest', 9, 'Lycia', 'Kill the fire-breathing Chimera terrorizing the land', 'King''s favor and land', 'completed', '1290-05-15 BC'),
    ('Journey to the Underworld', 9, 'Quest', 10, 'Underworld', 'Retrieve wife Eurydice from death', 'Failed - looked back', 'failed', NULL),
    ('Capture Cerberus', 1, 'Labor of Heracles', 10, 'Underworld', 'Capture the three-headed dog without weapons', 'Completion of labors', 'completed', '1200-12-25 BC'),
    ('Kill the Calydonian Boar', 8, 'Quest', 7, 'Calydon', 'Hunt and kill the monstrous boar', 'Boar''s hide and fame', 'completed', '1270-09-10 BC');

-- Insert legendary artifacts
INSERT INTO artifacts (name, type, forged_by_god_id, current_owner_id, owner_type, power_description, material, is_cursed) VALUES
    ('Zeus'' Lightning Bolt', 'Weapon', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Zeus'), 'god', 'Unlimited lightning strikes that can destroy anything', 'Divine Bronze', false),
    ('Poseidon''s Trident', 'Weapon', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Poseidon'), 'god', 'Control over seas, earthquakes, and creation of springs', 'Divine Bronze', false),
    ('Hades'' Helm of Darkness', 'Helmet', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Hades'), 'god', 'Grants complete invisibility to the wearer', 'Divine Metal', false),
    ('Aegis Shield', 'Shield', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Athena'), 'god', 'Athena''s shield bearing Medusa''s head, causes terror', 'Divine Bronze', false),
    ('Hermes'' Winged Sandals', 'Footwear', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Hermes'), 'god', 'Grants ability to fly at incredible speeds', 'Divine Leather', false),
    ('Golden Fleece', 'Artifact', NULL, 6, 'hero', 'Brings prosperity and healing to its owner', 'Golden Wool', false),
    ('Pandora''s Box', 'Container', (SELECT id FROM gods WHERE name = 'Hephaestus'), NULL, 'lost', 'Contains all evils of the world, only hope remains inside', 'Enchanted Wood', true),
    ('Bow of Heracles', 'Weapon', (SELECT id FROM gods WHERE name = 'Apollo'), 1, 'hero', 'Arrows never miss and are coated with Hydra poison', 'Divine Wood', false),
    ('Sword of Damocles', 'Weapon', NULL, NULL, 'lost', 'Represents imminent and ever-present peril', 'Steel', true),
    ('Ambrosia and Nectar', 'Food/Drink', NULL, NULL, 'god', 'Food and drink of gods, grants immortality', 'Divine Essence', false),
    ('Medusa''s Head', 'Artifact', NULL, 2, 'hero', 'Turns any who look upon it to stone', 'Petrified Gorgon', true),
    ('Thread of Ariadne', 'Tool', NULL, 5, 'hero', 'Magical thread that helped escape the labyrinth', 'Enchanted Thread', false),
    ('Orpheus'' Lyre', 'Instrument', (SELECT id FROM gods WHERE name = 'Apollo'), 9, 'hero', 'Music that charms all living things and even death itself', 'Divine Tortoise Shell', false),
    ('Talaria', 'Footwear', (SELECT id FROM gods WHERE name = 'Hephaestus'), 2, 'hero', 'Winged sandals granting flight, gift from Hermes', 'Divine Leather', false),
    ('Chariot of Helios', 'Vehicle', (SELECT id FROM gods WHERE name = 'Hephaestus'), (SELECT id FROM gods WHERE name = 'Helios'), 'god', 'Sun chariot that drives across the sky each day', 'Gold and Fire', false),
    ('Prometheus'' Torch', 'Artifact', NULL, (SELECT id FROM gods WHERE name = 'Prometheus'), 'god', 'The sacred fire stolen from the gods for humanity', 'Divine Flame', false);

-- Insert Titan artifacts
INSERT INTO artifacts (name, type, forged_by_titan_id, current_owner_id, owner_type, power_description, material, is_cursed) VALUES
    ('Scythe of Cronus', 'Weapon', NULL, (SELECT id FROM titans WHERE name = 'Cronus'), 'titan', 'Weapon used to overthrow Uranus, controls time', 'Adamantine', true);

-- Create indexes for better performance
CREATE INDEX idx_primordials_name ON primordials(name);
CREATE INDEX idx_titans_parent ON titans(parent_id);
CREATE INDEX idx_titans_name ON titans(name);
CREATE INDEX idx_gods_domain ON gods(domain);
CREATE INDEX idx_gods_is_olympian ON gods(is_olympian);
CREATE INDEX idx_gods_parent ON gods(parent_id);
CREATE INDEX idx_gods_parent_titan ON gods(parent_titan_id);
CREATE INDEX idx_heroes_patron_god ON heroes(patron_god_id);
CREATE INDEX idx_heroes_is_demigod ON heroes(is_demigod);
CREATE INDEX idx_creatures_type ON creatures(type);
CREATE INDEX idx_creatures_slain_by ON creatures(slain_by_hero_id);
CREATE INDEX idx_quests_hero ON quests(hero_id);
CREATE INDEX idx_quests_status ON quests(status);
CREATE INDEX idx_artifacts_owner ON artifacts(current_owner_id, owner_type);

-- Create a view for hero achievements
CREATE VIEW hero_achievements AS
SELECT
    h.name as hero_name,
    h.birth_place,
    g.name as patron_god,
    h.is_demigod,
    h.fame_level,
    COUNT(DISTINCT q.id) as quests_completed,
    COUNT(DISTINCT c.id) as monsters_slain,
    STRING_AGG(DISTINCT c.name, ', ') as defeated_creatures
FROM heroes h
LEFT JOIN gods g ON h.patron_god_id = g.id
LEFT JOIN quests q ON h.id = q.hero_id AND q.status = 'completed'
LEFT JOIN creatures c ON h.id = c.slain_by_hero_id
GROUP BY h.id, h.name, h.birth_place, g.name, h.is_demigod, h.fame_level;

-- Create a view for complete divine lineage
CREATE VIEW divine_lineage AS
-- Gods with god parents
SELECT
    child.name as entity_name,
    'God' as entity_type,
    child.domain,
    parent.name as parent_name,
    'God' as parent_type,
    parent.domain as parent_domain,
    child.is_olympian
FROM gods child
LEFT JOIN gods parent ON child.parent_id = parent.id
WHERE child.parent_id IS NOT NULL
UNION ALL
-- Gods with titan parents
SELECT
    child.name as entity_name,
    'God' as entity_type,
    child.domain,
    parent.name as parent_name,
    'Titan' as parent_type,
    parent.domain as parent_domain,
    child.is_olympian
FROM gods child
LEFT JOIN titans parent ON child.parent_titan_id = parent.id
WHERE child.parent_titan_id IS NOT NULL
UNION ALL
-- Titans with primordial parents
SELECT
    child.name as entity_name,
    'Titan' as entity_type,
    child.domain,
    parent.name as parent_name,
    'Primordial' as parent_type,
    parent.domain as parent_domain,
    false as is_olympian
FROM titans child
LEFT JOIN primordials parent ON child.parent_id = parent.id
WHERE child.parent_id IS NOT NULL;

-- Create a view for artifact ownership
CREATE VIEW artifact_registry AS
SELECT
    a.name as artifact_name,
    a.type,
    a.is_cursed,
    COALESCE(forger_god.name, forger_titan.name, 'Unknown') as forged_by,
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

-- Create a summary view of all entities
CREATE VIEW mythology_summary AS
SELECT 'Primordials' as category, COUNT(*) as count FROM primordials
UNION ALL
SELECT 'Titans' as category, COUNT(*) as count FROM titans
UNION ALL
SELECT 'Gods' as category, COUNT(*) as count FROM gods
UNION ALL
SELECT 'Heroes' as category, COUNT(*) as count FROM heroes
UNION ALL
SELECT 'Creatures' as category, COUNT(*) as count FROM creatures
UNION ALL
SELECT 'Quests' as category, COUNT(*) as count FROM quests
UNION ALL
SELECT 'Artifacts' as category, COUNT(*) as count FROM artifacts;
