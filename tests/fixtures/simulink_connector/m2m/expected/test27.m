if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'pp');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'pp', 'Folder', project_path);
end
cd(project_path);
MyInt = Simulink.NumericType;
MyInt.DataTypeMode = 'Fixed-point: slope and bias scaling';
MyInt.Bias = 0;
MyInt.Signedness = 'Signed';
MyInt.WordLength = 64;
MyInt.IsAlias = true;
assignin('base', 'MyInt', MyInt);
MyFloat = Simulink.NumericType;
MyFloat.DataTypeMode = 'Double';
MyFloat.IsAlias = true;
assignin('base', 'MyFloat', MyFloat);
if isfile('data/SA/SA_Data.sldd')
	SA_Data = Simulink.data.dictionary.open('data/SA/SA_Data.sldd');
else
	SA_Data = Simulink.data.dictionary.create('data/SA/SA_Data.sldd');
end
importFromBaseWorkspace(SA_Data, 'varList', {'MyInt', 'MyFloat'}, 'existingVarsAction','overwrite');
saveChanges(SA_Data);
clear SA_Data;
file = proj.addFile('data/SA/SA_Data.sldd');
addLabel(file, 'Capella Layer', 'SA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Property_1';
elems(1).Description = 'ModelerId:32fcc5b9-998b-41f5-b7a0-ab7a670ae7da';
elems(1).DataType = 'string';
elems(2) = Simulink.BusElement;
elems(2).Name = 'Property_2';
elems(2).Description = 'ModelerId:fd8adb57-67c7-45cf-b94f-0cb1514b9113';
elems(2).DataType = 'string';
busClass_Class_1 = Simulink.Bus;
busClass_Class_1.Description = 'ModelerId:fbcafd12-cf81-45cd-857a-794c20874805';
busClass_Class_1.Elements = elems;
assignin('base', 'busClass_Class_1', busClass_Class_1);
clear elems;
if isfile('data/SA/SA_DataPkg_classes.sldd')
	SA_DataPkg_classes = Simulink.data.dictionary.open('data/SA/SA_DataPkg_classes.sldd');
else
	SA_DataPkg_classes = Simulink.data.dictionary.create('data/SA/SA_DataPkg_classes.sldd');
end
importFromBaseWorkspace(SA_DataPkg_classes, 'varList', {'busClass_Class_1'}, 'existingVarsAction','overwrite');
saveChanges(SA_DataPkg_classes);
clear SA_DataPkg_classes;
file = proj.addFile('data/SA/SA_DataPkg_classes.sldd');
addLabel(file, 'Capella Layer', 'SA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItemElement_1';
elems(1).Description = 'ModelerId:1650a08e-2ca4-4946-971b-7984b030b7b5';
elems(1).DataType = 'Bus: busClass_Class_1';
busEI_ExchangeItem_1 = Simulink.Bus;
busEI_ExchangeItem_1.Description = 'ModelerId:a710cc48-1571-4f42-a304-a487c426c51f';
busEI_ExchangeItem_1.Elements = elems;
assignin('base', 'busEI_ExchangeItem_1', busEI_ExchangeItem_1);
clear elems;
if isfile('data/SA/SA_DataPkg_ei.sldd')
	SA_DataPkg_ei = Simulink.data.dictionary.open('data/SA/SA_DataPkg_ei.sldd');
else
	SA_DataPkg_ei = Simulink.data.dictionary.create('data/SA/SA_DataPkg_ei.sldd');
end
importFromBaseWorkspace(SA_DataPkg_ei, 'varList', {'busEI_ExchangeItem_1'}, 'existingVarsAction','overwrite');
addDataSource(SA_DataPkg_ei, 'SA_DataPkg_classes.sldd');
saveChanges(SA_DataPkg_ei);
clear SA_DataPkg_ei;
file = proj.addFile('data/SA/SA_DataPkg_ei.sldd');
addLabel(file, 'Capella Layer', 'SA');
evalin('base', "clear 'busClass_Class_1' 'MyFloat' 'busEI_ExchangeItem_1' 'MyInt'");
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;