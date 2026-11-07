new_system('buses');
add_block('simulink/Sources/Constant', 'buses/cst_1');
set_param('buses/cst_1', 'Position', '[10 10 35 35]')
set_param('buses/cst_1', 'value', '1')
handles = get_param('buses/cst_1', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'buses/cst_2');
set_param('buses/cst_2', 'Position', '[10 110 35 135]')
set_param('buses/cst_2', 'value', '2')
handles = get_param('buses/cst_2', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'buses/cst_3');
set_param('buses/cst_3', 'Position', '[10 210 35 235]')
set_param('buses/cst_3', 'value', '3')
handles = get_param('buses/cst_3', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'buses/cst_4');
set_param('buses/cst_4', 'Position', '[10 310 35 335]')
set_param('buses/cst_4', 'value', '4')
handles = get_param('buses/cst_4', 'porthandles');
clear handles;
add_block('simulink/Signal Routing/Bus Creator', 'buses/creator1');
set_param('buses/creator1', 'Position', '[100 70 110 120]')
handles = get_param('buses/creator1', 'porthandles');
clear handles;
add_block('simulink/Signal Routing/Bus Creator', 'buses/creator2');
set_param('buses/creator2', 'Position', '[200 140 210 190]')
set_param('buses/creator2', 'Inputs', '3')
handles = get_param('buses/creator2', 'porthandles');
clear handles;
add_block('simulink/Signal Routing/Bus Selector', 'buses/selector1');
set_param('buses/selector1', 'Position', '[400 140 410 190]')
set_param('buses/selector1', 'OutputSignals', 'signal1, signal2, signal3')
handles = get_param('buses/selector1', 'porthandles');
clear handles;
handles = get_param('buses/selector1', 'porthandles');
clear handles;
handles = get_param('buses/selector1', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'buses/display1');
set_param('buses/display1', 'Position', '[550 140 650 190]')
add_block('simulink/Sinks/Display', 'buses/display2');
set_param('buses/display2', 'Position', '[550 210 650 260]')
add_block('simulink/Sinks/Display', 'buses/display3');
set_param('buses/display3', 'Position', '[550 310 650 360]')
handles = get_param('buses/cst_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('buses/creator1', 'PortHandles');
target = handles.Inport(1);
line = add_line('buses', source, target, 'autorouting','smart');
set_param(line, 'Name', 'cst1____creator1');
clear line handles source target;
handles = get_param('buses/cst_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('buses/creator1', 'PortHandles');
target = handles.Inport(2);
line = add_line('buses', source, target, 'autorouting','smart');
set_param(line, 'Name', 'cst2____creator1');
clear line handles source target;
handles = get_param('buses/cst_3', 'PortHandles');
source = handles.Outport(1);
handles = get_param('buses/creator2', 'PortHandles');
target = handles.Inport(2);
line = add_line('buses', source, target, 'autorouting','smart');
set_param(line, 'Name', 'cst_3____creator2');
clear line handles source target;
handles = get_param('buses/cst_4', 'PortHandles');
source = handles.Outport(1);
handles = get_param('buses/creator2', 'PortHandles');
target = handles.Inport(3);
line = add_line('buses', source, target, 'autorouting','smart');
set_param(line, 'Name', 'cst4____creator2');
clear line handles source target;
handles = get_param('buses/creator1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('buses/creator2', 'PortHandles');
target = handles.Inport(1);
line = add_line('buses', source, target, 'autorouting','smart');
set_param(line, 'Name', 'creator1____creator2');
clear line handles source target;
handles = get_param('buses/creator2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('buses/selector1', 'PortHandles');
target = handles.Inport(1);
line = add_line('buses', source, target, 'autorouting','smart');
set_param(line, 'Name', 'creator2____selector');
clear line handles source target;
handles = get_param('buses/selector1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('buses/display1', 'PortHandles');
target = handles.Inport(1);
line = add_line('buses', source, target, 'autorouting','smart');
set_param(line, 'Name', 'selector____display1');
clear line handles source target;
handles = get_param('buses/selector1', 'PortHandles');
source = handles.Outport(2);
handles = get_param('buses/display2', 'PortHandles');
target = handles.Inport(1);
line = add_line('buses', source, target, 'autorouting','smart');
set_param(line, 'Name', 'selector____display2');
clear line handles source target;
handles = get_param('buses/selector1', 'PortHandles');
source = handles.Outport(3);
handles = get_param('buses/display3', 'PortHandles');
target = handles.Inport(1);
line = add_line('buses', source, target, 'autorouting','smart');
set_param(line, 'Name', 'selector____display3');
clear line handles source target;
save_system('buses', fullfile(project_path, 'models/buses.slx'));
