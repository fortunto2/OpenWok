-- Seed data: LA pilot expansion — 15 new restaurants across 6 zones, 80+ menu items
-- Uses deterministic IDs for D1 compatibility

-- Venice restaurants (3)
INSERT OR IGNORE INTO restaurants (id, name, zone_id, active) VALUES
    ('rest-brdsd-0001', 'Boardside Burgers', 'zone-vnce-0001', 1),
    ('rest-vnpzz-0001', 'Venice Pizza Co', 'zone-vnce-0001', 1),
    ('rest-abshl-0001', 'Abbot Kinney Bowls', 'zone-vnce-0001', 1);

-- Santa Monica restaurants (3)
INSERT OR IGNORE INTO restaurants (id, name, zone_id, active) VALUES
    ('rest-baysh-0001', 'Bay Sushi House', 'zone-smca-0001', 1),
    ('rest-prmdl-0001', 'Promenade Deli', 'zone-smca-0001', 1),
    ('rest-smtqr-0001', 'Santa Monica Taqueria', 'zone-smca-0001', 1);

-- Koreatown restaurants (3)
INSERT OR IGNORE INTO restaurants (id, name, zone_id, active) VALUES
    ('rest-soulq-0001', 'Seoul Q BBQ', 'zone-ktwn-0001', 1),
    ('rest-kbckn-0001', 'K-Bird Fried Chicken', 'zone-ktwn-0001', 1),
    ('rest-bngmn-0001', 'Bingsu Mountain', 'zone-ktwn-0001', 1);

-- Silver Lake restaurants (3)
INSERT OR IGNORE INTO restaurants (id, name, zone_id, active) VALUES
    ('rest-slkcf-0001', 'Silver Lake Coffee & Bites', 'zone-svlk-0001', 1),
    ('rest-snstb-0001', 'Sunset Blvd Thai', 'zone-svlk-0001', 1),
    ('rest-hpstr-0001', 'Hyperion Street Eats', 'zone-svlk-0001', 1);

-- Downtown LA additional (2)
INSERT OR IGNORE INTO restaurants (id, name, zone_id, active) VALUES
    ('rest-grndm-0001', 'Grand Market Noodles', 'zone-dtla-0001', 1),
    ('rest-artsm-0001', 'Arts District Smokehouse', 'zone-dtla-0001', 1);

-- Hollywood additional (1)
INSERT OR IGNORE INTO restaurants (id, name, zone_id, active) VALUES
    ('rest-snstp-0001', 'Sunset Strip Poke', 'zone-hlwd-0001', 1);

-- Menu items: Venice
INSERT OR IGNORE INTO menu_items (id, restaurant_id, name, price) VALUES
    ('item-brd-001', 'rest-brdsd-0001', 'Classic Smash Burger', '11.99'),
    ('item-brd-002', 'rest-brdsd-0001', 'Truffle Fries', '7.49'),
    ('item-brd-003', 'rest-brdsd-0001', 'BBQ Bacon Burger', '14.99'),
    ('item-brd-004', 'rest-brdsd-0001', 'Milkshake', '6.99'),
    ('item-brd-005', 'rest-brdsd-0001', 'Chicken Sandwich', '12.49'),
    ('item-vnp-001', 'rest-vnpzz-0001', 'Margherita Pizza', '15.99'),
    ('item-vnp-002', 'rest-vnpzz-0001', 'Pepperoni Pizza', '17.99'),
    ('item-vnp-003', 'rest-vnpzz-0001', 'Caesar Salad', '9.99'),
    ('item-vnp-004', 'rest-vnpzz-0001', 'Garlic Knots', '5.99'),
    ('item-vnp-005', 'rest-vnpzz-0001', 'Tiramisu', '8.99'),
    ('item-abk-001', 'rest-abshl-0001', 'Acai Bowl', '13.99'),
    ('item-abk-002', 'rest-abshl-0001', 'Poke Bowl', '15.99'),
    ('item-abk-003', 'rest-abshl-0001', 'Smoothie Bowl', '11.99'),
    ('item-abk-004', 'rest-abshl-0001', 'Grain Bowl', '14.49'),
    ('item-abk-005', 'rest-abshl-0001', 'Avocado Toast', '10.99');

-- Menu items: Santa Monica
INSERT OR IGNORE INTO menu_items (id, restaurant_id, name, price) VALUES
    ('item-bay-001', 'rest-baysh-0001', 'Omakase Roll', '18.99'),
    ('item-bay-002', 'rest-baysh-0001', 'Salmon Sashimi', '16.99'),
    ('item-bay-003', 'rest-baysh-0001', 'Spicy Tuna Roll', '14.99'),
    ('item-bay-004', 'rest-baysh-0001', 'Edamame', '5.99'),
    ('item-bay-005', 'rest-baysh-0001', 'Miso Ramen', '15.99'),
    ('item-bay-006', 'rest-baysh-0001', 'Gyoza', '8.99'),
    ('item-prm-001', 'rest-prmdl-0001', 'Turkey Club', '13.99'),
    ('item-prm-002', 'rest-prmdl-0001', 'Reuben Sandwich', '14.99'),
    ('item-prm-003', 'rest-prmdl-0001', 'Matzo Ball Soup', '9.99'),
    ('item-prm-004', 'rest-prmdl-0001', 'Bagel & Lox', '12.99'),
    ('item-prm-005', 'rest-prmdl-0001', 'Pastrami on Rye', '15.49'),
    ('item-smt-001', 'rest-smtqr-0001', 'Carne Asada Tacos', '11.99'),
    ('item-smt-002', 'rest-smtqr-0001', 'Fish Tacos', '12.99'),
    ('item-smt-003', 'rest-smtqr-0001', 'Burrito Supreme', '13.99'),
    ('item-smt-004', 'rest-smtqr-0001', 'Elote', '4.99'),
    ('item-smt-005', 'rest-smtqr-0001', 'Horchata', '3.99');

-- Menu items: Koreatown
INSERT OR IGNORE INTO menu_items (id, restaurant_id, name, price) VALUES
    ('item-slq-001', 'rest-soulq-0001', 'Galbi Set', '24.99'),
    ('item-slq-002', 'rest-soulq-0001', 'Bulgogi', '19.99'),
    ('item-slq-003', 'rest-soulq-0001', 'Japchae', '13.99'),
    ('item-slq-004', 'rest-soulq-0001', 'Kimchi Jjigae', '14.99'),
    ('item-slq-005', 'rest-soulq-0001', 'Bibimbap', '16.99'),
    ('item-slq-006', 'rest-soulq-0001', 'Korean Fried Tofu', '10.99'),
    ('item-kbc-001', 'rest-kbckn-0001', 'Crispy Chicken Combo', '14.99'),
    ('item-kbc-002', 'rest-kbckn-0001', 'Spicy Wings', '12.99'),
    ('item-kbc-003', 'rest-kbckn-0001', 'Chicken Sandwich', '11.99'),
    ('item-kbc-004', 'rest-kbckn-0001', 'Tteokbokki', '9.99'),
    ('item-kbc-005', 'rest-kbckn-0001', 'Corn Cheese', '7.99'),
    ('item-bng-001', 'rest-bngmn-0001', 'Mango Bingsu', '12.99'),
    ('item-bng-002', 'rest-bngmn-0001', 'Matcha Bingsu', '13.99'),
    ('item-bng-003', 'rest-bngmn-0001', 'Red Bean Bingsu', '11.99'),
    ('item-bng-004', 'rest-bngmn-0001', 'Hotteok', '6.99'),
    ('item-bng-005', 'rest-bngmn-0001', 'Taro Milk Tea', '5.99');

-- Menu items: Silver Lake
INSERT OR IGNORE INTO menu_items (id, restaurant_id, name, price) VALUES
    ('item-slc-001', 'rest-slkcf-0001', 'Avocado Egg Sandwich', '11.99'),
    ('item-slc-002', 'rest-slkcf-0001', 'Oat Milk Latte', '5.99'),
    ('item-slc-003', 'rest-slkcf-0001', 'Banana Bread', '4.49'),
    ('item-slc-004', 'rest-slkcf-0001', 'Breakfast Burrito', '10.99'),
    ('item-slc-005', 'rest-slkcf-0001', 'Granola Parfait', '8.99'),
    ('item-snb-001', 'rest-snstb-0001', 'Pad See Ew', '13.99'),
    ('item-snb-002', 'rest-snstb-0001', 'Massaman Curry', '15.99'),
    ('item-snb-003', 'rest-snstb-0001', 'Larb Gai', '12.99'),
    ('item-snb-004', 'rest-snstb-0001', 'Mango Sticky Rice', '8.99'),
    ('item-snb-005', 'rest-snstb-0001', 'Thai Iced Tea', '4.99'),
    ('item-hps-001', 'rest-hpstr-0001', 'Nashville Hot Chicken', '14.99'),
    ('item-hps-002', 'rest-hpstr-0001', 'Mac & Cheese', '8.99'),
    ('item-hps-003', 'rest-hpstr-0001', 'Loaded Fries', '9.99'),
    ('item-hps-004', 'rest-hpstr-0001', 'Pulled Pork Sandwich', '13.49'),
    ('item-hps-005', 'rest-hpstr-0001', 'Coleslaw', '4.99');

-- Menu items: Downtown LA (additional restaurants)
INSERT OR IGNORE INTO menu_items (id, restaurant_id, name, price) VALUES
    ('item-grn-001', 'rest-grndm-0001', 'Beef Pho', '14.99'),
    ('item-grn-002', 'rest-grndm-0001', 'Dan Dan Noodles', '12.99'),
    ('item-grn-003', 'rest-grndm-0001', 'Wonton Soup', '10.99'),
    ('item-grn-004', 'rest-grndm-0001', 'Char Siu Bao', '7.99'),
    ('item-grn-005', 'rest-grndm-0001', 'Shrimp Dumplings', '11.99'),
    ('item-art-001', 'rest-artsm-0001', 'Brisket Plate', '19.99'),
    ('item-art-002', 'rest-artsm-0001', 'Pulled Pork Plate', '16.99'),
    ('item-art-003', 'rest-artsm-0001', 'Smoked Ribs Half Rack', '22.99'),
    ('item-art-004', 'rest-artsm-0001', 'Cornbread', '4.99'),
    ('item-art-005', 'rest-artsm-0001', 'Baked Beans', '5.99');

-- Menu items: Hollywood (additional restaurant)
INSERT OR IGNORE INTO menu_items (id, restaurant_id, name, price) VALUES
    ('item-ssp-001', 'rest-snstp-0001', 'Ahi Poke Bowl', '16.99'),
    ('item-ssp-002', 'rest-snstp-0001', 'Salmon Poke Bowl', '15.99'),
    ('item-ssp-003', 'rest-snstp-0001', 'Tofu Poke Bowl', '13.99'),
    ('item-ssp-004', 'rest-snstp-0001', 'Seaweed Salad', '6.99'),
    ('item-ssp-005', 'rest-snstp-0001', 'Coconut Water', '3.99');
