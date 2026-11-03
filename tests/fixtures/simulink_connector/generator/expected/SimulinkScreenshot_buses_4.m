if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'SimulinkScreenshot_buses_3');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'SimulinkScreenshot_buses_3', 'Folder', project_path);
end
cd(project_path);
ID = Simulink.NumericType;
ID.DataTypeMode = 'Fixed-point: slope and bias scaling';
ID.Bias = 0;
ID.Signedness = 'Signed';
ID.WordLength = 64;
ID.IsAlias = true;
assignin('base', 'ID', ID);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Value';
elems(1).DataType = 'double';
elems(2) = Simulink.BusElement;
elems(2).Name = 'Validity';
elems(2).DataType = 'boolean';
elems(3) = Simulink.BusElement;
elems(3).Name = 'busId';
elems(3).DataType = 'Bus: mySubBus';
myBus = Simulink.Bus;
myBus.Elements = elems;
assignin('base', 'myBus', myBus);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Id';
elems(1).DataType = 'ID';
mySubBus = Simulink.Bus;
mySubBus.Elements = elems;
assignin('base', 'mySubBus', mySubBus);
clear elems;
if isfile('data/bus__1_.sldd')
	bus__1_ = Simulink.data.dictionary.open('data/bus__1_.sldd');
else
	bus__1_ = Simulink.data.dictionary.create('data/bus__1_.sldd');
end
importFromBaseWorkspace(bus__1_, 'varList', {'ID', 'myBus', 'mySubBus'}, 'existingVarsAction','overwrite');
saveChanges(bus__1_);
clear bus__1_;
file = proj.addFile('data/bus__1_.sldd');
evalin('base', "clear 'myBus' 'mySubBus' 'ID'");
new_system('MyProject');
set_param('MyProject', 'DataDictionary', 'bus__1_.sldd');
add_block('simulink/Ports & Subsystems/Subsystem', 'MyProject/System_A');
set_param('MyProject/System_A', 'Position', '[300 20 380 100]')
CapSLBridge.internal.cleanSubsystem('MyProject/System_A');
add_block('simulink/Ports & Subsystems/In1', 'MyProject/System_A/in1');
set_param('MyProject/System_A/in1', 'Port', '1');
set_param('MyProject/System_A/in1', 'Position', '[1 1 31 21]');
add_block('simulink/Ports & Subsystems/Subsystem', 'MyProject/System_B');
set_param('MyProject/System_B', 'Position', '[40 20 120 100]')
CapSLBridge.internal.cleanSubsystem('MyProject/System_B');
add_block('simulink/Ports & Subsystems/Out1', 'MyProject/System_B/out1');
set_param('MyProject/System_B/out1', 'Port', '1');
set_param('MyProject/System_B/out1', 'Position', '[380 30 410 50]');
set_param('MyProject/System_B/out1', 'OutDataTypeStr', 'Bus: myBus')
add_block('simulink/Sources/Constant', 'MyProject/System_B/MyValue');
set_param('MyProject/System_B/MyValue', 'Position', '[10 10 40 40]')
set_param('MyProject/System_B/MyValue', 'OutDataTypeStr', 'double')
set_param('MyProject/System_B/MyValue', 'value', '100')
handles = get_param('MyProject/System_B/MyValue', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MyProject/System_B/MyValidity');
set_param('MyProject/System_B/MyValidity', 'Position', '[10 100 40 130]')
set_param('MyProject/System_B/MyValidity', 'OutDataTypeStr', 'boolean')
set_param('MyProject/System_B/MyValidity', 'value', 'true')
handles = get_param('MyProject/System_B/MyValidity', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MyProject/System_B/MyID');
set_param('MyProject/System_B/MyID', 'Position', '[10 180 40 210]')
set_param('MyProject/System_B/MyID', 'OutDataTypeStr', 'ID')
set_param('MyProject/System_B/MyID', 'Value', '1')
handles = get_param('MyProject/System_B/MyID', 'porthandles');
clear handles;
add_block('simulink/Signal Routing/Bus Creator', 'MyProject/System_B/myBus_creator');
set_param('MyProject/System_B/myBus_creator', 'Position', '[300 10 310 100]')
set_param('MyProject/System_B/myBus_creator', 'OutDataTypeStr', 'Bus: myBus')
set_param('MyProject/System_B/myBus_creator', 'Inputs', '3')
handles = get_param('MyProject/System_B/myBus_creator', 'porthandles');
clear handles;
add_block('simulink/Signal Routing/Bus Creator', 'MyProject/System_B/mySubBus_creator');
set_param('MyProject/System_B/mySubBus_creator', 'Position', '[200 100 210 130]')
set_param('MyProject/System_B/mySubBus_creator', 'OutDataTypeStr', 'Bus: mySubBus')
set_param('MyProject/System_B/mySubBus_creator', 'Inputs', '1')
handles = get_param('MyProject/System_B/mySubBus_creator', 'porthandles');
clear handles;
handles = get_param('MyProject/System_B/MyValue', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_B/myBus_creator', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyProject/System_B', source, target, 'autorouting','smart');
set_param(line, 'Name', 'value____creator');
clear line handles source target;
handles = get_param('MyProject/System_B/MyValidity', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_B/myBus_creator', 'PortHandles');
target = handles.Inport(2);
line = add_line('MyProject/System_B', source, target, 'autorouting','smart');
set_param(line, 'Name', 'validity____creator');
clear line handles source target;
handles = get_param('MyProject/System_B/myBus_creator', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_B/out1', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyProject/System_B', source, target, 'autorouting','smart');
set_param(line, 'Name', 'creator____port');
clear line handles source target;
handles = get_param('MyProject/System_B/MyID', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_B/mySubBus_creator', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyProject/System_B', source, target, 'autorouting','smart');
set_param(line, 'Name', 'id____sub_creator');
clear line handles source target;
handles = get_param('MyProject/System_B/mySubBus_creator', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_B/myBus_creator', 'PortHandles');
target = handles.Inport(3);
line = add_line('MyProject/System_B', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sub_creator____creator');
clear line handles source target;
handles = get_param('MyProject/System_B', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MyProject/System_A', 'PortHandles');
target = handles.Inport(1);
line = add_line('MyProject', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sys_A____sys_B');
clear line handles source target;
save_system('MyProject', fullfile(project_path, 'models/MyProject.slx'));
file = proj.addFile('models/MyProject.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
