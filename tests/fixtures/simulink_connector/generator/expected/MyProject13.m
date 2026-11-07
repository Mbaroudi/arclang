if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject13');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject13', 'Folder', project_path);
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
new_system('MySystem13');
set_param('MySystem13', 'DataDictionary', 'bus__1_.sldd');
add_block('simulink/Sources/Constant', 'MySystem13/MyCst1');
set_param('MySystem13/MyCst1', 'Position', '[10 10 40 40]')
set_param('MySystem13/MyCst1', 'OutDataTypeStr', 'Bus: myBus');
set_param('MySystem13/MyCst1', 'value', '0')
handles = get_param('MySystem13/MyCst1', 'porthandles');
clear handles;
add_block('simulink/Sources/Constant', 'MySystem13/MyCst2');
set_param('MySystem13/MyCst2', 'Position', '[10 100 40 130]')
set_param('MySystem13/MyCst2', 'value', '5')
handles = get_param('MySystem13/MyCst2', 'porthandles');
clear handles;
add_block('simulink/Sinks/Display', 'MySystem13/Display');
set_param('MySystem13/Display', 'Position', '[350 20 400 40]')
add_block('simulink/Ports & Subsystems/Subsystem', 'MySystem13/MySystem');
set_param('MySystem13/MySystem', 'Position', '[200 20 280 100]')
CapSLBridge.internal.cleanSubsystem('MySystem13/MySystem');
add_block('simulink/Ports & Subsystems/In1', 'MySystem13/MySystem/in1');
set_param('MySystem13/MySystem/in1', 'Port', '1');
set_param('MySystem13/MySystem/in1', 'Position', '[10 60 40 80]');
add_block('simulink/Ports & Subsystems/Out1', 'MySystem13/MySystem/out1');
set_param('MySystem13/MySystem/out1', 'Port', '1');
set_param('MySystem13/MySystem/out1', 'OutDataTypeStr', 'Bus: myBus');
set_param('MySystem13/MySystem/out1', 'BusOutputAsStruct', 'on');
set_param('MySystem13/MySystem/out1', 'Position', '[100 110 130 130]');
add_block('simulink/Ports & Subsystems/In1', 'MySystem13/MySystem/in2');
set_param('MySystem13/MySystem/in2', 'Port', '2');
set_param('MySystem13/MySystem/in2', 'OutDataTypeStr', 'Bus: myBus');
set_param('MySystem13/MySystem/in2', 'Position', '[10 110 40 130]');
add_block('simulink/Ports & Subsystems/In1', 'MySystem13/MySystem/in3');
set_param('MySystem13/MySystem/in3', 'Port', '3');
set_param('MySystem13/MySystem/in3', 'Position', '[10 160 40 180]');
handles = get_param('MySystem13/MyCst1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MySystem13/MySystem', 'PortHandles');
target = handles.Inport(2);
line = add_line('MySystem13', source, target, 'autorouting','smart');
set_param(line, 'Name', '1____sys');
clear line handles source target;
handles = get_param('MySystem13/MyCst2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('MySystem13/MySystem', 'PortHandles');
target = handles.Inport(3);
line = add_line('MySystem13', source, target, 'autorouting','smart');
set_param(line, 'Name', '2____sys');
clear line handles source target;
save_system('MySystem13', fullfile(project_path, 'models/MySystem13.slx'));
file = proj.addFile('models/MySystem13.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
