-- Seed data: LA pilot — 2 zones, 3 restaurants, 9 menu items

INSERT OR IGNORE INTO zones (id, name) VALUES
    ('zone-dtla-0001', 'Downtown LA'),
    ('zone-hlwd-0001', 'Hollywood');

INSERT OR IGNORE INTO restaurants (id, name, zone_id, active) VALUES
    ('rest-padthai-0001', 'Pad Thai Palace', 'zone-dtla-0001', 1),
    ('rest-sushi-0001', 'Sushi Wave', 'zone-dtla-0001', 1),
    ('rest-taco-0001', 'Taco Libre', 'zone-hlwd-0001', 1);

INSERT OR IGNORE INTO menu_items (id, restaurant_id, name, price) VALUES
    ('item-pt-001', 'rest-padthai-0001', 'Pad Thai', '12.99'),
    ('item-pt-002', 'rest-padthai-0001', 'Tom Yum Soup', '8.99'),
    ('item-pt-003', 'rest-padthai-0001', 'Green Curry', '14.99'),
    ('item-sw-001', 'rest-sushi-0001', 'California Roll', '10.99'),
    ('item-sw-002', 'rest-sushi-0001', 'Salmon Nigiri', '13.99'),
    ('item-sw-003', 'rest-sushi-0001', 'Miso Soup', '4.99'),
    ('item-tl-001', 'rest-taco-0001', 'Street Tacos', '9.99'),
    ('item-tl-002', 'rest-taco-0001', 'Burrito Bowl', '11.99'),
    ('item-tl-003', 'rest-taco-0001', 'Churros', '5.99');
