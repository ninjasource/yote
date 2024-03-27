All vias (0.5/0.3mm and 0.3/0.15mm) to be filled and capped (because 0.3/0.15mm via-in-pad used for 0.4mm pitch bga components).
Panelization: PCWWay to please panlize board to 2 x 3 and increase the panel size accordingly.
Silkscreen: It is acceptable to not show Silkscreen on non-printable areas.
Please confirm rotation of components in a visual way with me to make sure it is correct (silkscreen should provide markers and CGI render can be used too).
Min track / spacing: 3mil
Min hole size: 0.15mm

Impedance control:

The track that connects the MCU to the antenna needs to be impedance matched to 50 ohms. Here are the calculations:

Used this calculator for surface microstrip for antenna line:
https://www.pcbway.com/pcb_prototype/impedance_calculator.html

w      = 0.134mm (calculated track width)
t Cu   = 35um (track height - I included the plating - should I have?)
h      = 0.08mm (isolation height)
Er     = 4.29 (Dialectric constant - FR4 - Standard 4.3)
Z0     = 50.0 ohm (Impedance)
Layer  = L1 (Front - yote-F_Cu.gbr)
Type   = Single ended
Pos    = x 19.45 y 6.85 (the short transmission line that L2 sits on going from the nrf5340 mcu to the antenna)
Tollerance = 10%


Used this stackup tool:
https://www.pcbway.com/multi-layer-laminated-structure.html?tdsourcetag=s_pcqq_aiomsg

Board Stackup:

Use PCBWay defaults for 8 layer board 1mm thickness
Number of Layers:   8
Board Thickness:    1 mm
Copper thickness:   1 oz

Stackup:

L1	 	Copper 18 um--plating to 35um
 	 	 
 	 	PP 0.08 mm(1080) dielectric constant 4.29 ± (The DK value is not absolute and will vary depending on the base material's models and thickness.)
 	 	 
L2	 	 
 	 	Core 0.2mm with 1/1 oz Cu
L3	 	 
 	 	 
 	 	PP 0.08 mm(1080) dielectric constant 4.29 ± (The DK value is not absolute and will vary depending on the base material's models and thickness.)
 	 	 
L4	 	 
 	 	Core 0.2mm with 1/1 oz Cu
L5	 	 
 	 	 
 	 	PP 0.08 mm(1080) dielectric constant 4.29 ± (The DK value is not absolute and will vary depending on the base material's models and thickness.)
 	 	 
L6	 	 
 	 	Core 0.2mm with 1/1 oz Cu
L7	 	 
 	 	 
 	 	PP 0.08 mm(1080) dielectric constant 4.29 ± (The DK value is not absolute and will vary depending on the base material's models and thickness.)
 	 	 
L8	 	Copper 18 um--plating to 35um
