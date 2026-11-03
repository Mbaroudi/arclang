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
PhysicalQuantity_1 = Simulink.Signal;
PhysicalQuantity_1.Unit = 'm';
PhysicalQuantity_1.Description = 'ModelerId:ff9ea6c7-6ce1-41b4-9e65-c3cfe738740f';
assignin('base', 'PhysicalQuantity_1', PhysicalQuantity_1);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Property_1';
elems(1).Description = 'ModelerId:600749cb-c43c-4ba7-9ac1-c1e910ef08ae';
busClass_Class_1 = Simulink.Bus;
busClass_Class_1.Description = 'ModelerId:51958f33-18c2-49ed-b782-210331341a85';
busClass_Class_1.Elements = elems;
assignin('base', 'busClass_Class_1', busClass_Class_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItemElement_1';
elems(1).Description = 'ModelerId:cd3f50bc-fb9f-48db-a389-5bffe74e78f5';
elems(1).DataType = 'Bus: busClass_Class_1';
busEI_ExchangeItem_1 = Simulink.Bus;
busEI_ExchangeItem_1.Description = 'ModelerId:040a0bdd-a53c-468b-94f1-6f7297808a82';
busEI_ExchangeItem_1.Elements = elems;
assignin('base', 'busEI_ExchangeItem_1', busEI_ExchangeItem_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItemElement_1';
elems(1).Description = 'ModelerId:3b9b0538-0732-44ab-8930-02bfd50f7510';
elems(1).DataType = 'Bus: busClass_Class_1';
busEI_ExchangeItem_2 = Simulink.Bus;
busEI_ExchangeItem_2.Description = 'ModelerId:ae5e8cac-d3ce-400d-8b19-f51a9cb74278';
busEI_ExchangeItem_2.Elements = elems;
assignin('base', 'busEI_ExchangeItem_2', busEI_ExchangeItem_2);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'sub_ExchangeItemElement_1';
elems(1).Description = 'ModelerId:14443d90-d032-4a53-90b5-248cd67306a3';
elems(1).DataType = 'Bus: busClass_Class_1';
busEI_sub_ExchangeItem_1 = Simulink.Bus;
busEI_sub_ExchangeItem_1.Description = 'ModelerId:97d4d41f-e227-46f4-8cff-7316b2a316ef';
busEI_sub_ExchangeItem_1.Elements = elems;
assignin('base', 'busEI_sub_ExchangeItem_1', busEI_sub_ExchangeItem_1);
clear elems;
if isfile('data/PA/PA_Data.sldd')
	PA_Data = Simulink.data.dictionary.open('data/PA/PA_Data.sldd');
else
	PA_Data = Simulink.data.dictionary.create('data/PA/PA_Data.sldd');
end
importFromBaseWorkspace(PA_Data, 'varList', {'PhysicalQuantity_1', 'busClass_Class_1', 'busEI_ExchangeItem_1', 'busEI_ExchangeItem_2', 'busEI_sub_ExchangeItem_1'}, 'existingVarsAction','overwrite');
saveChanges(PA_Data);
clear PA_Data;
file = proj.addFile('data/PA/PA_Data.sldd');
addLabel(file, 'Capella Layer', 'PA');
clear elems;
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:be094c00-f0b3-440a-a0c2-1e7febcd20df';
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
clear elems;
busFE_FunctionalExchange_2 = Simulink.Bus;
busFE_FunctionalExchange_2.Description = 'ModelerId:8aaa3039-dc85-46ef-a201-4d26e7873993';
assignin('base', 'busFE_FunctionalExchange_2', busFE_FunctionalExchange_2);
clear elems;
clear elems;
busFE_leaf1_to_bottom1 = Simulink.Bus;
busFE_leaf1_to_bottom1.Description = 'ModelerId:f29a19c0-7886-4b9b-9ff0-df0bb36dbcb8';
assignin('base', 'busFE_leaf1_to_bottom1', busFE_leaf1_to_bottom1);
clear elems;
clear elems;
busFE_bottom2_to_leaf2 = Simulink.Bus;
busFE_bottom2_to_leaf2.Description = 'ModelerId:48620e46-8811-4368-99a4-4d6401053a1e';
assignin('base', 'busFE_bottom2_to_leaf2', busFE_bottom2_to_leaf2);
clear elems;
clear elems;
busFE_FunctionalExchange_4 = Simulink.Bus;
busFE_FunctionalExchange_4.Description = 'ModelerId:4020f037-79ae-4ffb-be04-9ce7acc43a2e';
assignin('base', 'busFE_FunctionalExchange_4', busFE_FunctionalExchange_4);
clear elems;
clear elems;
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:40af6851-833b-46fc-add0-95e6d433ab9f';
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
clear elems;
busFE_FunctionalExchange_5 = Simulink.Bus;
busFE_FunctionalExchange_5.Description = 'ModelerId:4e8e5e1e-e889-4221-8c1a-3af14c740db5';
assignin('base', 'busFE_FunctionalExchange_5', busFE_FunctionalExchange_5);
clear elems;
clear elems;
busFE_FunctionalExchange_2 = Simulink.Bus;
busFE_FunctionalExchange_2.Description = 'ModelerId:632b6ef3-c35a-41d2-997f-f80170fff7d4';
assignin('base', 'busFE_FunctionalExchange_2', busFE_FunctionalExchange_2);
clear elems;
clear elems;
busFE_FunctionalExchange_3 = Simulink.Bus;
busFE_FunctionalExchange_3.Description = 'ModelerId:46b3c1db-b44d-4667-bc1f-6a8d3b60fab2';
assignin('base', 'busFE_FunctionalExchange_3', busFE_FunctionalExchange_3);
clear elems;
clear elems;
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:6e911035-42a1-434b-b1c3-f9c696de00e8';
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
clear elems;
busFE_sub = Simulink.Bus;
busFE_sub.Description = 'ModelerId:a06d23f6-6296-4637-93b3-1780ead25ffc';
assignin('base', 'busFE_sub', busFE_sub);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_1';
elems(1).Description = 'ModelerId:040a0bdd-a53c-468b-94f1-6f7297808a82';
elems(1).DataType = 'Bus: busEI_ExchangeItem_1';
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:9d5f8800-df02-4681-86c0-5680b7ebd444';
busFE_FunctionalExchange_1.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_2';
elems(1).Description = 'ModelerId:ae5e8cac-d3ce-400d-8b19-f51a9cb74278';
elems(1).DataType = 'Bus: busEI_ExchangeItem_2';
busFE_FunctionalExchange_2 = Simulink.Bus;
busFE_FunctionalExchange_2.Description = 'ModelerId:8f69952d-36e4-46ae-bef9-d5dcd8138483';
busFE_FunctionalExchange_2.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_2', busFE_FunctionalExchange_2);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'sub_ExchangeItem_1';
elems(1).Description = 'ModelerId:97d4d41f-e227-46f4-8cff-7316b2a316ef';
elems(1).DataType = 'Bus: busEI_sub_ExchangeItem_1';
busFE_Sub_FunctionalExchange_1 = Simulink.Bus;
busFE_Sub_FunctionalExchange_1.Description = 'ModelerId:566e5d9f-f6bc-468a-9fd5-ded0f4a10c51';
busFE_Sub_FunctionalExchange_1.Elements = elems;
assignin('base', 'busFE_Sub_FunctionalExchange_1', busFE_Sub_FunctionalExchange_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'sub_ExchangeItem_1';
elems(1).Description = 'ModelerId:97d4d41f-e227-46f4-8cff-7316b2a316ef';
elems(1).DataType = 'Bus: busEI_sub_ExchangeItem_1';
busFE_sub_FunctionalExchange_1 = Simulink.Bus;
busFE_sub_FunctionalExchange_1.Description = 'ModelerId:65a7b34a-9a23-47f3-81b7-d22c644c694d';
busFE_sub_FunctionalExchange_1.Elements = elems;
assignin('base', 'busFE_sub_FunctionalExchange_1', busFE_sub_FunctionalExchange_1);
clear elems;
clear elems;
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:8e52be06-ff6f-4fb2-b496-571d8ac36692';
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
if isfile('data/PA/PA_FunctionalExchanges_Categories.sldd')
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/PA/PA_FunctionalExchanges_Categories.sldd');
else
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/PA/PA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(PA_FunctionalExchanges_Categories, 'varList', {'busFE_FunctionalExchange_1', 'busFE_FunctionalExchange_2', 'busFE_leaf1_to_bottom1', 'busFE_bottom2_to_leaf2', 'busFE_FunctionalExchange_4', 'busFE_FunctionalExchange_1', 'busFE_FunctionalExchange_5', 'busFE_FunctionalExchange_2', 'busFE_FunctionalExchange_3', 'busFE_FunctionalExchange_1', 'busFE_sub', 'busFE_FunctionalExchange_1', 'busFE_FunctionalExchange_2', 'busFE_Sub_FunctionalExchange_1', 'busFE_sub_FunctionalExchange_1', 'busFE_FunctionalExchange_1'}, 'existingVarsAction','overwrite');
addDataSource(PA_FunctionalExchanges_Categories, 'PA_Data.sldd');
saveChanges(PA_FunctionalExchanges_Categories);
clear PA_FunctionalExchanges_Categories;
file = proj.addFile('data/PA/PA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Root_Physical_Function.sldd')
	PA_Root_Physical_Function = Simulink.data.dictionary.open('data/PA/PA_Root_Physical_Function.sldd');
else
	PA_Root_Physical_Function = Simulink.data.dictionary.create('data/PA/PA_Root_Physical_Function.sldd');
end
addDataSource(PA_Root_Physical_Function, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Root_Physical_Function);
clear PA_Root_Physical_Function;
file = proj.addFile('data/PA/PA_Root_Physical_Function.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf_1.sldd')
	PA_leaf_1 = Simulink.data.dictionary.open('data/PA/PA_leaf_1.sldd');
else
	PA_leaf_1 = Simulink.data.dictionary.create('data/PA/PA_leaf_1.sldd');
end
addDataSource(PA_leaf_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf_1);
clear PA_leaf_1;
file = proj.addFile('data/PA/PA_leaf_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Leaf_2.sldd')
	PA_Leaf_2 = Simulink.data.dictionary.open('data/PA/PA_Leaf_2.sldd');
else
	PA_Leaf_2 = Simulink.data.dictionary.create('data/PA/PA_Leaf_2.sldd');
end
addDataSource(PA_Leaf_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Leaf_2);
clear PA_Leaf_2;
file = proj.addFile('data/PA/PA_Leaf_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf_1.sldd')
	PA_leaf_1 = Simulink.data.dictionary.open('data/PA/PA_leaf_1.sldd');
else
	PA_leaf_1 = Simulink.data.dictionary.create('data/PA/PA_leaf_1.sldd');
end
addDataSource(PA_leaf_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf_1);
clear PA_leaf_1;
file = proj.addFile('data/PA/PA_leaf_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf_2.sldd')
	PA_leaf_2 = Simulink.data.dictionary.open('data/PA/PA_leaf_2.sldd');
else
	PA_leaf_2 = Simulink.data.dictionary.create('data/PA/PA_leaf_2.sldd');
end
addDataSource(PA_leaf_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf_2);
clear PA_leaf_2;
file = proj.addFile('data/PA/PA_leaf_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf1.sldd')
	PA_leaf1 = Simulink.data.dictionary.open('data/PA/PA_leaf1.sldd');
else
	PA_leaf1 = Simulink.data.dictionary.create('data/PA/PA_leaf1.sldd');
end
addDataSource(PA_leaf1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf1);
clear PA_leaf1;
file = proj.addFile('data/PA/PA_leaf1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf2.sldd')
	PA_leaf2 = Simulink.data.dictionary.open('data/PA/PA_leaf2.sldd');
else
	PA_leaf2 = Simulink.data.dictionary.create('data/PA/PA_leaf2.sldd');
end
addDataSource(PA_leaf2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf2);
clear PA_leaf2;
file = proj.addFile('data/PA/PA_leaf2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Leaf_4.sldd')
	PA_Leaf_4 = Simulink.data.dictionary.open('data/PA/PA_Leaf_4.sldd');
else
	PA_Leaf_4 = Simulink.data.dictionary.create('data/PA/PA_Leaf_4.sldd');
end
addDataSource(PA_Leaf_4, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Leaf_4);
clear PA_Leaf_4;
file = proj.addFile('data/PA/PA_Leaf_4.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Container_1.sldd')
	PA_Container_1 = Simulink.data.dictionary.open('data/PA/PA_Container_1.sldd');
else
	PA_Container_1 = Simulink.data.dictionary.create('data/PA/PA_Container_1.sldd');
end
addDataSource(PA_Container_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Container_1);
clear PA_Container_1;
file = proj.addFile('data/PA/PA_Container_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Leaf_1.sldd')
	PA_Leaf_1 = Simulink.data.dictionary.open('data/PA/PA_Leaf_1.sldd');
else
	PA_Leaf_1 = Simulink.data.dictionary.create('data/PA/PA_Leaf_1.sldd');
end
addDataSource(PA_Leaf_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Leaf_1);
clear PA_Leaf_1;
file = proj.addFile('data/PA/PA_Leaf_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Leaf_2.sldd')
	PA_Leaf_2 = Simulink.data.dictionary.open('data/PA/PA_Leaf_2.sldd');
else
	PA_Leaf_2 = Simulink.data.dictionary.create('data/PA/PA_Leaf_2.sldd');
end
addDataSource(PA_Leaf_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Leaf_2);
clear PA_Leaf_2;
file = proj.addFile('data/PA/PA_Leaf_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Leaf_3.sldd')
	PA_Leaf_3 = Simulink.data.dictionary.open('data/PA/PA_Leaf_3.sldd');
else
	PA_Leaf_3 = Simulink.data.dictionary.create('data/PA/PA_Leaf_3.sldd');
end
addDataSource(PA_Leaf_3, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Leaf_3);
clear PA_Leaf_3;
file = proj.addFile('data/PA/PA_Leaf_3.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Container_2.sldd')
	PA_Container_2 = Simulink.data.dictionary.open('data/PA/PA_Container_2.sldd');
else
	PA_Container_2 = Simulink.data.dictionary.create('data/PA/PA_Container_2.sldd');
end
addDataSource(PA_Container_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Container_2);
clear PA_Container_2;
file = proj.addFile('data/PA/PA_Container_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf1.sldd')
	PA_leaf1 = Simulink.data.dictionary.open('data/PA/PA_leaf1.sldd');
else
	PA_leaf1 = Simulink.data.dictionary.create('data/PA/PA_leaf1.sldd');
end
addDataSource(PA_leaf1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf1);
clear PA_leaf1;
file = proj.addFile('data/PA/PA_leaf1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_pf_1.sldd')
	PA_pf_1 = Simulink.data.dictionary.open('data/PA/PA_pf_1.sldd');
else
	PA_pf_1 = Simulink.data.dictionary.create('data/PA/PA_pf_1.sldd');
end
addDataSource(PA_pf_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_pf_1);
clear PA_pf_1;
file = proj.addFile('data/PA/PA_pf_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_pf_2.sldd')
	PA_pf_2 = Simulink.data.dictionary.open('data/PA/PA_pf_2.sldd');
else
	PA_pf_2 = Simulink.data.dictionary.create('data/PA/PA_pf_2.sldd');
end
addDataSource(PA_pf_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_pf_2);
clear PA_pf_2;
file = proj.addFile('data/PA/PA_pf_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_sub_1.sldd')
	PA_sub_1 = Simulink.data.dictionary.open('data/PA/PA_sub_1.sldd');
else
	PA_sub_1 = Simulink.data.dictionary.create('data/PA/PA_sub_1.sldd');
end
addDataSource(PA_sub_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_sub_1);
clear PA_sub_1;
file = proj.addFile('data/PA/PA_sub_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_sub_2.sldd')
	PA_sub_2 = Simulink.data.dictionary.open('data/PA/PA_sub_2.sldd');
else
	PA_sub_2 = Simulink.data.dictionary.create('data/PA/PA_sub_2.sldd');
end
addDataSource(PA_sub_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_sub_2);
clear PA_sub_2;
file = proj.addFile('data/PA/PA_sub_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_sub_1.sldd')
	PA_sub_1 = Simulink.data.dictionary.open('data/PA/PA_sub_1.sldd');
else
	PA_sub_1 = Simulink.data.dictionary.create('data/PA/PA_sub_1.sldd');
end
addDataSource(PA_sub_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_sub_1);
clear PA_sub_1;
file = proj.addFile('data/PA/PA_sub_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_bottom2.sldd')
	PA_bottom2 = Simulink.data.dictionary.open('data/PA/PA_bottom2.sldd');
else
	PA_bottom2 = Simulink.data.dictionary.create('data/PA/PA_bottom2.sldd');
end
addDataSource(PA_bottom2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_bottom2);
clear PA_bottom2;
file = proj.addFile('data/PA/PA_bottom2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_bottom_1.sldd')
	PA_bottom_1 = Simulink.data.dictionary.open('data/PA/PA_bottom_1.sldd');
else
	PA_bottom_1 = Simulink.data.dictionary.create('data/PA/PA_bottom_1.sldd');
end
addDataSource(PA_bottom_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_bottom_1);
clear PA_bottom_1;
file = proj.addFile('data/PA/PA_bottom_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_bottom1.sldd')
	PA_bottom1 = Simulink.data.dictionary.open('data/PA/PA_bottom1.sldd');
else
	PA_bottom1 = Simulink.data.dictionary.create('data/PA/PA_bottom1.sldd');
end
addDataSource(PA_bottom1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_bottom1);
clear PA_bottom1;
file = proj.addFile('data/PA/PA_bottom1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf2.sldd')
	PA_leaf2 = Simulink.data.dictionary.open('data/PA/PA_leaf2.sldd');
else
	PA_leaf2 = Simulink.data.dictionary.create('data/PA/PA_leaf2.sldd');
end
addDataSource(PA_leaf2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf2);
clear PA_leaf2;
file = proj.addFile('data/PA/PA_leaf2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf3.sldd')
	PA_leaf3 = Simulink.data.dictionary.open('data/PA/PA_leaf3.sldd');
else
	PA_leaf3 = Simulink.data.dictionary.create('data/PA/PA_leaf3.sldd');
end
addDataSource(PA_leaf3, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf3);
clear PA_leaf3;
file = proj.addFile('data/PA/PA_leaf3.sldd');
addLabel(file, 'Capella Layer', 'PA');
evalin('base', "clear 'PhysicalQuantity_1' 'busClass_Class_1' 'busFE_sub' 'busEI_ExchangeItem_1' 'busEI_ExchangeItem_2' 'busEI_sub_ExchangeItem_1' 'busFE_bottom2_to_leaf2' 'busFE_FunctionalExchange_1' 'busFE_leaf1_to_bottom1' 'busFE_sub_FunctionalExchange_1' 'busFE_FunctionalExchange_3' 'busFE_FunctionalExchange_2' 'busFE_FunctionalExchange_5' 'busFE_FunctionalExchange_4' 'busFE_Sub_FunctionalExchange_1'");
new_system('PA_leaf3');
set_param('PA_leaf3', 'Tag', 'ModelerId:03f7e903-d08f-4375-8c0f-964f1ed50ec5');
set_param('PA_leaf3', 'DataDictionary', 'PA_leaf3.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_leaf3/FunctionalExchange_1');
set_param('PA_leaf3/FunctionalExchange_1', 'Port', '1');
set_param('PA_leaf3/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_leaf3/FunctionalExchange_1', 'Tag', 'ModelerId:2d99ce12-ffe0-4139-b3c6-63bbda0978ab');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_leaf3/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_leaf3/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_leaf3', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf3');
catch
end
save_system('PA_leaf3', fullfile(project_path, 'models/PA/PA_leaf3.slx'));
file = proj.addFile('models/PA/PA_leaf3.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf2');
set_param('PA_leaf2', 'Tag', 'ModelerId:82c22b93-0695-45f9-a80e-07244eaced6b');
set_param('PA_leaf2', 'DataDictionary', 'PA_leaf2.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_leaf2/FunctionalExchange_1');
set_param('PA_leaf2/FunctionalExchange_1', 'Port', '1');
set_param('PA_leaf2/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_leaf2/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_leaf2/FunctionalExchange_1', 'Tag', 'ModelerId:74707e01-4261-4020-ad5e-b0cefc568ed0');
g_Block = add_block('simulink/Sources/Ground', 'PA_leaf2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_leaf2/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_leaf2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf2');
catch
end
save_system('PA_leaf2', fullfile(project_path, 'models/PA/PA_leaf2.slx'));
file = proj.addFile('models/PA/PA_leaf2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_bottom1');
set_param('PA_bottom1', 'Tag', 'ModelerId:0d18653e-f4cb-4b57-9d99-6fa587a4634f');
set_param('PA_bottom1', 'DataDictionary', 'PA_bottom1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_bottom1/leaf1_to_bottom1');
set_param('PA_bottom1/leaf1_to_bottom1', 'Port', '1');
set_param('PA_bottom1/leaf1_to_bottom1', 'OutDataTypeStr', 'Bus: busFE_leaf1_to_bottom1');
set_param('PA_bottom1/leaf1_to_bottom1', 'Tag', 'ModelerId:1c92e414-4dc0-445a-9cfa-419fa214911f');
add_block('simulink/Ports & Subsystems/Out1', 'PA_bottom1/sub');
set_param('PA_bottom1/sub', 'Port', '1');
set_param('PA_bottom1/sub', 'OutDataTypeStr', 'Bus: busFE_sub');
set_param('PA_bottom1/sub', 'BusOutputAsStruct', 'on');
set_param('PA_bottom1/sub', 'Tag', 'ModelerId:395722eb-7184-475d-81dc-21e78f051c2b');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_bottom1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_bottom1/leaf1_to_bottom1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_bottom1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
g_Block = add_block('simulink/Sources/Ground', 'PA_bottom1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_bottom1/sub', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_bottom1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_bottom1');
catch
end
save_system('PA_bottom1', fullfile(project_path, 'models/PA/PA_bottom1.slx'));
file = proj.addFile('models/PA/PA_bottom1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_bottom_1');
set_param('PA_bottom_1', 'Tag', 'ModelerId:18d24771-6019-4037-bbd9-8be358085c80');
set_param('PA_bottom_1', 'DataDictionary', 'PA_bottom_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_bottom_1/FunctionalExchange_2');
set_param('PA_bottom_1/FunctionalExchange_2', 'Port', '1');
set_param('PA_bottom_1/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_bottom_1/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_bottom_1/FunctionalExchange_2', 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
add_block('simulink/Ports & Subsystems/In1', 'PA_bottom_1/sub_FunctionalExchange_1');
set_param('PA_bottom_1/sub_FunctionalExchange_1', 'Port', '1');
set_param('PA_bottom_1/sub_FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_sub_FunctionalExchange_1');
set_param('PA_bottom_1/sub_FunctionalExchange_1', 'Tag', 'ModelerId:92dfd0ca-0b71-4235-b595-9c326ec57e82');
g_Block = add_block('simulink/Sources/Ground', 'PA_bottom_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_bottom_1/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_bottom_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
t_Block = add_block('simulink/Sinks/Terminator', 'PA_bottom_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_bottom_1/sub_FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_bottom_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_bottom_1');
catch
end
save_system('PA_bottom_1', fullfile(project_path, 'models/PA/PA_bottom_1.slx'));
file = proj.addFile('models/PA/PA_bottom_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_bottom2');
set_param('PA_bottom2', 'Tag', 'ModelerId:ffa504fd-8e29-4095-8e15-dd41f7e08583');
set_param('PA_bottom2', 'DataDictionary', 'PA_bottom2.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_bottom2/bottom2_to_leaf2');
set_param('PA_bottom2/bottom2_to_leaf2', 'Port', '1');
set_param('PA_bottom2/bottom2_to_leaf2', 'OutDataTypeStr', 'Bus: busFE_bottom2_to_leaf2');
set_param('PA_bottom2/bottom2_to_leaf2', 'BusOutputAsStruct', 'on');
set_param('PA_bottom2/bottom2_to_leaf2', 'Tag', 'ModelerId:9de1a4fc-6543-4647-8fd3-60098c5dabe0');
add_block('simulink/Ports & Subsystems/In1', 'PA_bottom2/sub');
set_param('PA_bottom2/sub', 'Port', '1');
set_param('PA_bottom2/sub', 'OutDataTypeStr', 'Bus: busFE_sub');
set_param('PA_bottom2/sub', 'Tag', 'ModelerId:0bc182b2-abb3-4b9d-93f4-ccc61967c15d');
g_Block = add_block('simulink/Sources/Ground', 'PA_bottom2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_bottom2/bottom2_to_leaf2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_bottom2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
t_Block = add_block('simulink/Sinks/Terminator', 'PA_bottom2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_bottom2/sub', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_bottom2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_bottom2');
catch
end
save_system('PA_bottom2', fullfile(project_path, 'models/PA/PA_bottom2.slx'));
file = proj.addFile('models/PA/PA_bottom2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_sub_1');
set_param('PA_sub_1', 'Tag', 'ModelerId:003b2df1-c09c-48e7-9811-5b9563057a96');
set_param('PA_sub_1', 'DataDictionary', 'PA_sub_1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_sub_1/FunctionalExchange_1');
set_param('PA_sub_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_sub_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_sub_1/FunctionalExchange_1', 'Tag', 'ModelerId:260b2635-00bc-43f8-a1b1-12f639e39480');
add_block('simulink/Ports & Subsystems/Out1', 'PA_sub_1/sub_FunctionalExchange_1');
set_param('PA_sub_1/sub_FunctionalExchange_1', 'Port', '1');
set_param('PA_sub_1/sub_FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_sub_FunctionalExchange_1');
set_param('PA_sub_1/sub_FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_sub_1/sub_FunctionalExchange_1', 'Tag', 'ModelerId:b4ad6c0a-7966-420f-bb2f-696a36b99c08');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_sub_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_sub_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_sub_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
g_Block = add_block('simulink/Sources/Ground', 'PA_sub_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_sub_1/sub_FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_sub_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_sub_1');
catch
end
save_system('PA_sub_1', fullfile(project_path, 'models/PA/PA_sub_1.slx'));
file = proj.addFile('models/PA/PA_sub_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_sub_2');
set_param('PA_sub_2', 'Tag', 'ModelerId:ee137cc4-d52d-484a-b685-16c449142824');
set_param('PA_sub_2', 'DataDictionary', 'PA_sub_2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_sub_2/Sub_FunctionalExchange_1');
set_param('PA_sub_2/Sub_FunctionalExchange_1', 'Port', '1');
set_param('PA_sub_2/Sub_FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_Sub_FunctionalExchange_1');
set_param('PA_sub_2/Sub_FunctionalExchange_1', 'Tag', 'ModelerId:79348269-33bd-4522-a8fb-21afeddd74bf');
add_block('simulink/Ports & Subsystems/Out1', 'PA_sub_2/FunctionalExchange_2');
set_param('PA_sub_2/FunctionalExchange_2', 'Port', '1');
set_param('PA_sub_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_sub_2/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_sub_2/FunctionalExchange_2', 'Tag', 'ModelerId:171fe0da-d7f1-4b89-874b-3fcce7204f85');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_sub_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_sub_2/Sub_FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_sub_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
g_Block = add_block('simulink/Sources/Ground', 'PA_sub_2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_sub_2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_sub_2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_sub_2');
catch
end
save_system('PA_sub_2', fullfile(project_path, 'models/PA/PA_sub_2.slx'));
file = proj.addFile('models/PA/PA_sub_2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_sub_1');
set_param('PA_sub_1', 'Tag', 'ModelerId:9d68ddf2-be8b-42f2-a28c-729f7fe949d1');
set_param('PA_sub_1', 'DataDictionary', 'PA_sub_1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_sub_1/FunctionalExchange_1');
set_param('PA_sub_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_sub_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_sub_1/FunctionalExchange_1', 'Tag', 'ModelerId:fb45b6cd-452e-48e5-94b8-a2c634d36d84');
add_block('simulink/Ports & Subsystems/Out1', 'PA_sub_1/Sub_FunctionalExchange_1');
set_param('PA_sub_1/Sub_FunctionalExchange_1', 'Port', '1');
set_param('PA_sub_1/Sub_FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_Sub_FunctionalExchange_1');
set_param('PA_sub_1/Sub_FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_sub_1/Sub_FunctionalExchange_1', 'Tag', 'ModelerId:8efaf64b-a45d-4cfc-9f40-1be624687e15');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_sub_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_sub_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_sub_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
g_Block = add_block('simulink/Sources/Ground', 'PA_sub_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_sub_1/Sub_FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_sub_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_sub_1');
catch
end
save_system('PA_sub_1', fullfile(project_path, 'models/PA/PA_sub_1.slx'));
file = proj.addFile('models/PA/PA_sub_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_pf_2');
set_param('PA_pf_2', 'Tag', 'ModelerId:093e62cb-9bb3-4141-8507-4229af11d8e0');
set_param('PA_pf_2', 'DataDictionary', 'PA_pf_2.sldd');
add_block('simulink/Ports & Subsystems/Enable', 'PA_pf_2/EN_pf_2');
set_param('PA_pf_2/EN_pf_2', 'Description', 'State 1');
set_param('PA_pf_2/EN_pf_2', 'AttributesFormatString', 'Available in Modes & States: %<Description>')
set_param('PA_pf_2/EN_pf_2', 'ShowName', 'off')
add_block('simulink/Ports & Subsystems/In1', 'PA_pf_2/FunctionalExchange_1');
set_param('PA_pf_2/FunctionalExchange_1', 'Port', '1');
set_param('PA_pf_2/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_pf_2/FunctionalExchange_1', 'Tag', 'ModelerId:62831711-028f-41ec-9dce-5a036630b9c4');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_pf_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_pf_2/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_pf_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_pf_2');
catch
end
save_system('PA_pf_2', fullfile(project_path, 'models/PA/PA_pf_2.slx'));
file = proj.addFile('models/PA/PA_pf_2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_pf_1');
set_param('PA_pf_1', 'Tag', 'ModelerId:36ece6e6-ae73-47b2-b9b3-9e102f6cee51');
set_param('PA_pf_1', 'DataDictionary', 'PA_pf_1.sldd');
add_block('simulink/Ports & Subsystems/Enable', 'PA_pf_1/EN_pf_1');
set_param('PA_pf_1/EN_pf_1', 'Description', 'State 1');
set_param('PA_pf_1/EN_pf_1', 'AttributesFormatString', 'Available in Modes & States: %<Description>')
set_param('PA_pf_1/EN_pf_1', 'ShowName', 'off')
add_block('simulink/Ports & Subsystems/Out1', 'PA_pf_1/FunctionalExchange_1');
set_param('PA_pf_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_pf_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_pf_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_pf_1/FunctionalExchange_1', 'Tag', 'ModelerId:d12d6e5e-326b-40f5-9dbf-8b98a7511d75');
g_Block = add_block('simulink/Sources/Ground', 'PA_pf_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_pf_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_pf_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_pf_1');
catch
end
save_system('PA_pf_1', fullfile(project_path, 'models/PA/PA_pf_1.slx'));
file = proj.addFile('models/PA/PA_pf_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf1');
set_param('PA_leaf1', 'Tag', 'ModelerId:81c8a58d-1798-4581-8e63-73e9b5e67f35');
set_param('PA_leaf1', 'DataDictionary', 'PA_leaf1.sldd');
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf1');
catch
end
save_system('PA_leaf1', fullfile(project_path, 'models/PA/PA_leaf1.slx'));
file = proj.addFile('models/PA/PA_leaf1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Container_2');
set_param('PA_Container_2', 'Tag', 'ModelerId:e7b08005-c709-4963-9826-d6188a34887f');
set_param('PA_Container_2', 'DataDictionary', 'PA_Container_2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Container_2/FunctionalExchange_2');
set_param('PA_Container_2/FunctionalExchange_2', 'Port', '1');
set_param('PA_Container_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_Container_2/FunctionalExchange_2', 'Tag', 'ModelerId:74132534-ba77-45c2-a616-ddd95d67480d');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Container_2/FunctionalExchange_3');
set_param('PA_Container_2/FunctionalExchange_3', 'Port', '1');
set_param('PA_Container_2/FunctionalExchange_3', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_3');
set_param('PA_Container_2/FunctionalExchange_3', 'BusOutputAsStruct', 'on');
set_param('PA_Container_2/FunctionalExchange_3', 'Tag', 'ModelerId:b71f74d4-36d7-4987-853a-dd60711e53f0');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Container_2/FunctionalExchange_4');
set_param('PA_Container_2/FunctionalExchange_4', 'Port', '2');
set_param('PA_Container_2/FunctionalExchange_4', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_4');
set_param('PA_Container_2/FunctionalExchange_4', 'BusOutputAsStruct', 'on');
set_param('PA_Container_2/FunctionalExchange_4', 'Tag', 'ModelerId:92b2407a-330e-4007-97d7-60dd613d4f2e');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Container_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Container_2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Container_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
g_Block = add_block('simulink/Sources/Ground', 'PA_Container_2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_Container_2/FunctionalExchange_3', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_Container_2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
g_Block = add_block('simulink/Sources/Ground', 'PA_Container_2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_Container_2/FunctionalExchange_4', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_Container_2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Container_2');
catch
end
save_system('PA_Container_2', fullfile(project_path, 'models/PA/PA_Container_2.slx'));
file = proj.addFile('models/PA/PA_Container_2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Leaf_3');
set_param('PA_Leaf_3', 'Tag', 'ModelerId:56c1f50d-ebad-4103-968f-25baf7557a2b');
set_param('PA_Leaf_3', 'DataDictionary', 'PA_Leaf_3.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Leaf_3/FunctionalExchange_5');
set_param('PA_Leaf_3/FunctionalExchange_5', 'Port', '1');
set_param('PA_Leaf_3/FunctionalExchange_5', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_5');
set_param('PA_Leaf_3/FunctionalExchange_5', 'BusOutputAsStruct', 'on');
set_param('PA_Leaf_3/FunctionalExchange_5', 'Tag', 'ModelerId:225d55f5-aa29-4389-97e4-feb621c8556f');
g_Block = add_block('simulink/Sources/Ground', 'PA_Leaf_3/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_Leaf_3/FunctionalExchange_5', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_Leaf_3', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Leaf_3');
catch
end
save_system('PA_Leaf_3', fullfile(project_path, 'models/PA/PA_Leaf_3.slx'));
file = proj.addFile('models/PA/PA_Leaf_3.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Leaf_2');
set_param('PA_Leaf_2', 'Tag', 'ModelerId:b25c4b83-b531-41ff-87f8-a8b686dba851');
set_param('PA_Leaf_2', 'DataDictionary', 'PA_Leaf_2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Leaf_2/FunctionalExchange_3');
set_param('PA_Leaf_2/FunctionalExchange_3', 'Port', '1');
set_param('PA_Leaf_2/FunctionalExchange_3', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_3');
set_param('PA_Leaf_2/FunctionalExchange_3', 'Tag', 'ModelerId:cb03c618-d503-41f1-bf4f-758c67514f24');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Leaf_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Leaf_2/FunctionalExchange_3', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Leaf_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Leaf_2');
catch
end
save_system('PA_Leaf_2', fullfile(project_path, 'models/PA/PA_Leaf_2.slx'));
file = proj.addFile('models/PA/PA_Leaf_2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Leaf_1');
set_param('PA_Leaf_1', 'Tag', 'ModelerId:5afe0c00-0313-479a-976f-b7771b500c94');
set_param('PA_Leaf_1', 'DataDictionary', 'PA_Leaf_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Leaf_1/FunctionalExchange_1');
set_param('PA_Leaf_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_Leaf_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_Leaf_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_Leaf_1/FunctionalExchange_1', 'Tag', 'ModelerId:24869e7d-82a3-4486-89e0-d4b7ab52ca54');
g_Block = add_block('simulink/Sources/Ground', 'PA_Leaf_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_Leaf_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_Leaf_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Leaf_1');
catch
end
save_system('PA_Leaf_1', fullfile(project_path, 'models/PA/PA_Leaf_1.slx'));
file = proj.addFile('models/PA/PA_Leaf_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Container_1');
set_param('PA_Container_1', 'Tag', 'ModelerId:385587e8-9c47-4d4c-9576-f35641c14163');
set_param('PA_Container_1', 'DataDictionary', 'PA_Container_1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Container_1/FunctionalExchange_1');
set_param('PA_Container_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_Container_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_Container_1/FunctionalExchange_1', 'Tag', 'ModelerId:6022bc1d-a918-4046-a5c3-add21b5551ba');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Container_1/FunctionalExchange_2');
set_param('PA_Container_1/FunctionalExchange_2', 'Port', '1');
set_param('PA_Container_1/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_Container_1/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_Container_1/FunctionalExchange_2', 'Tag', 'ModelerId:fc925960-f543-4069-9182-7b2f4c2e0351');
add_block('simulink/Ports & Subsystems/In1', 'PA_Container_1/FunctionalExchange_5');
set_param('PA_Container_1/FunctionalExchange_5', 'Port', '2');
set_param('PA_Container_1/FunctionalExchange_5', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_5');
set_param('PA_Container_1/FunctionalExchange_5', 'Tag', 'ModelerId:e910048a-c4fa-4e91-97e0-753b847d3018');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Container_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Container_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Container_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Container_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Container_1/FunctionalExchange_5', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Container_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
g_Block = add_block('simulink/Sources/Ground', 'PA_Container_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_Container_1/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_Container_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Container_1');
catch
end
save_system('PA_Container_1', fullfile(project_path, 'models/PA/PA_Container_1.slx'));
file = proj.addFile('models/PA/PA_Container_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Leaf_4');
set_param('PA_Leaf_4', 'Tag', 'ModelerId:b873a449-7c4e-4427-a059-a90dd4315e30');
set_param('PA_Leaf_4', 'DataDictionary', 'PA_Leaf_4.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Leaf_4/FunctionalExchange_4');
set_param('PA_Leaf_4/FunctionalExchange_4', 'Port', '1');
set_param('PA_Leaf_4/FunctionalExchange_4', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_4');
set_param('PA_Leaf_4/FunctionalExchange_4', 'Tag', 'ModelerId:45d9eef1-7e39-4df5-b99a-622df6fee4e6');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Leaf_4/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Leaf_4/FunctionalExchange_4', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Leaf_4', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Leaf_4');
catch
end
save_system('PA_Leaf_4', fullfile(project_path, 'models/PA/PA_Leaf_4.slx'));
file = proj.addFile('models/PA/PA_Leaf_4.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf2');
set_param('PA_leaf2', 'Tag', 'ModelerId:ee115b68-09f3-482f-9584-282bab1f6864');
set_param('PA_leaf2', 'DataDictionary', 'PA_leaf2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_leaf2/bottom2_to_leaf2');
set_param('PA_leaf2/bottom2_to_leaf2', 'Port', '1');
set_param('PA_leaf2/bottom2_to_leaf2', 'OutDataTypeStr', 'Bus: busFE_bottom2_to_leaf2');
set_param('PA_leaf2/bottom2_to_leaf2', 'Tag', 'ModelerId:16b7bebe-fe34-4f96-b62c-2968eb28dd4b');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_leaf2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_leaf2/bottom2_to_leaf2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_leaf2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf2');
catch
end
save_system('PA_leaf2', fullfile(project_path, 'models/PA/PA_leaf2.slx'));
file = proj.addFile('models/PA/PA_leaf2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf1');
set_param('PA_leaf1', 'Tag', 'ModelerId:1e5ed3de-ecc9-4902-ba70-71eeb58c021b');
set_param('PA_leaf1', 'DataDictionary', 'PA_leaf1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_leaf1/leaf1_to_bottom1');
set_param('PA_leaf1/leaf1_to_bottom1', 'Port', '1');
set_param('PA_leaf1/leaf1_to_bottom1', 'OutDataTypeStr', 'Bus: busFE_leaf1_to_bottom1');
set_param('PA_leaf1/leaf1_to_bottom1', 'BusOutputAsStruct', 'on');
set_param('PA_leaf1/leaf1_to_bottom1', 'Tag', 'ModelerId:9cb0754d-bd10-4738-b67e-21590601f767');
g_Block = add_block('simulink/Sources/Ground', 'PA_leaf1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_leaf1/leaf1_to_bottom1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_leaf1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf1');
catch
end
save_system('PA_leaf1', fullfile(project_path, 'models/PA/PA_leaf1.slx'));
file = proj.addFile('models/PA/PA_leaf1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf_2');
set_param('PA_leaf_2', 'Tag', 'ModelerId:178afa0b-0f08-41d8-ac7f-9db851bdf4bf');
set_param('PA_leaf_2', 'DataDictionary', 'PA_leaf_2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_leaf_2/FunctionalExchange_2');
set_param('PA_leaf_2/FunctionalExchange_2', 'Port', '1');
set_param('PA_leaf_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_leaf_2/FunctionalExchange_2', 'Tag', 'ModelerId:905fbdf8-70c6-407e-b6af-646c6a0f4a33');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_leaf_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_leaf_2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_leaf_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf_2');
catch
end
save_system('PA_leaf_2', fullfile(project_path, 'models/PA/PA_leaf_2.slx'));
file = proj.addFile('models/PA/PA_leaf_2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf_1');
set_param('PA_leaf_1', 'Tag', 'ModelerId:2df54e0f-10fc-4498-a73d-5ef1a75ee70a');
set_param('PA_leaf_1', 'DataDictionary', 'PA_leaf_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_leaf_1/FunctionalExchange_1');
set_param('PA_leaf_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_leaf_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_leaf_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_leaf_1/FunctionalExchange_1', 'Tag', 'ModelerId:29dd7f04-b10f-41f0-8bd5-cb04a7d20f17');
g_Block = add_block('simulink/Sources/Ground', 'PA_leaf_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_leaf_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_leaf_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf_1');
catch
end
save_system('PA_leaf_1', fullfile(project_path, 'models/PA/PA_leaf_1.slx'));
file = proj.addFile('models/PA/PA_leaf_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Leaf_2');
set_param('PA_Leaf_2', 'Tag', 'ModelerId:4cadc4a0-eed1-4ed8-a5ef-e85838cc70ee');
set_param('PA_Leaf_2', 'DataDictionary', 'PA_Leaf_2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Leaf_2/FunctionalExchange_2');
set_param('PA_Leaf_2/FunctionalExchange_2', 'Port', '1');
set_param('PA_Leaf_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_Leaf_2/FunctionalExchange_2', 'Tag', 'ModelerId:6540787d-e351-4218-b9a0-ee9ffec589ca');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_Leaf_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_Leaf_2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_Leaf_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_Leaf_2');
catch
end
save_system('PA_Leaf_2', fullfile(project_path, 'models/PA/PA_Leaf_2.slx'));
file = proj.addFile('models/PA/PA_Leaf_2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf_1');
set_param('PA_leaf_1', 'Tag', 'ModelerId:ffbf1867-6cbe-4c5b-ab0e-514ee01d5201');
set_param('PA_leaf_1', 'DataDictionary', 'PA_leaf_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_leaf_1/FunctionalExchange_1');
set_param('PA_leaf_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_leaf_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_leaf_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_leaf_1/FunctionalExchange_1', 'Tag', 'ModelerId:4a3f6acd-cb1d-4d89-b2d9-46d9d59f36e9');
g_Block = add_block('simulink/Sources/Ground', 'PA_leaf_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_leaf_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_leaf_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf_1');
catch
end
save_system('PA_leaf_1', fullfile(project_path, 'models/PA/PA_leaf_1.slx'));
file = proj.addFile('models/PA/PA_leaf_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Root_Physical_Function');
set_param('PA_Root_Physical_Function', 'Tag', 'ModelerId:3a6f178a-0017-4695-9069-593033054809');
set_param('PA_Root_Physical_Function', 'DataDictionary', 'PA_Root_Physical_Function.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Root_Physical_Function/EN_pf_1');
set_param('PA_Root_Physical_Function/EN_pf_1', 'Port', '1');
add_block('simulink/Ports & Subsystems/In1', 'PA_Root_Physical_Function/EN_pf_2');
set_param('PA_Root_Physical_Function/EN_pf_2', 'Port', '2');
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top');
set_param('PA_Root_Physical_Function/Top', 'Tag', 'ModelerId:3d71da20-b8cb-4baf-b08a-5884be4e28b4');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top/Reference_to_leaf_1');
set_param('PA_Root_Physical_Function/Top/Reference_to_leaf_1', 'Tag', 'ModelerId:ffbf1867-6cbe-4c5b-ab0e-514ee01d5201');
set_param('PA_Root_Physical_Function/Top/Reference_to_leaf_1', 'ModelFile', 'PA_leaf_1')
handles = get_param('PA_Root_Physical_Function/Top/Reference_to_leaf_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:4a3f6acd-cb1d-4d89-b2d9-46d9d59f36e9');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top/Reference_to_Leaf_2');
set_param('PA_Root_Physical_Function/Top/Reference_to_Leaf_2', 'Tag', 'ModelerId:4cadc4a0-eed1-4ed8-a5ef-e85838cc70ee');
set_param('PA_Root_Physical_Function/Top/Reference_to_Leaf_2', 'ModelFile', 'PA_Leaf_2')
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top/container_1');
set_param('PA_Root_Physical_Function/Top/container_1', 'Tag', 'ModelerId:fd35433b-16dc-4898-a915-fff7b41c426e');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top/container_1');
add_block('simulink/Ports & Subsystems/In1', 'PA_Root_Physical_Function/Top/container_1/FunctionalExchange_1');
set_param('PA_Root_Physical_Function/Top/container_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_Root_Physical_Function/Top/container_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_Root_Physical_Function/Top/container_1/FunctionalExchange_1', 'Tag', 'ModelerId:fb45b6cd-452e-48e5-94b8-a2c634d36d84');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Root_Physical_Function/Top/container_1/FunctionalExchange_2');
set_param('PA_Root_Physical_Function/Top/container_1/FunctionalExchange_2', 'Port', '1');
set_param('PA_Root_Physical_Function/Top/container_1/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_Root_Physical_Function/Top/container_1/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_Root_Physical_Function/Top/container_1/FunctionalExchange_2', 'Tag', 'ModelerId:171fe0da-d7f1-4b89-874b-3fcce7204f85');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top/container_1/Reference_to_sub_1');
set_param('PA_Root_Physical_Function/Top/container_1/Reference_to_sub_1', 'Tag', 'ModelerId:9d68ddf2-be8b-42f2-a28c-729f7fe949d1');
set_param('PA_Root_Physical_Function/Top/container_1/Reference_to_sub_1', 'ModelFile', 'PA_sub_1')
handles = get_param('PA_Root_Physical_Function/Top/container_1/Reference_to_sub_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:8efaf64b-a45d-4cfc-9f40-1be624687e15');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top/container_1/Reference_to_sub_2');
set_param('PA_Root_Physical_Function/Top/container_1/Reference_to_sub_2', 'Tag', 'ModelerId:ee137cc4-d52d-484a-b685-16c449142824');
set_param('PA_Root_Physical_Function/Top/container_1/Reference_to_sub_2', 'ModelFile', 'PA_sub_2')
handles = get_param('PA_Root_Physical_Function/Top/container_1/Reference_to_sub_2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:171fe0da-d7f1-4b89-874b-3fcce7204f85');
clear handles;
handles = get_param('PA_Root_Physical_Function/Top/container_1/FunctionalExchange_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top/container_1/Reference_to_sub_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top/container_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:fb45b6cd-452e-48e5-94b8-a2c634d36d84');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top/container_1/Reference_to_sub_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top/container_1/FunctionalExchange_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top/container_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:171fe0da-d7f1-4b89-874b-3fcce7204f85');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top/container_1/Reference_to_sub_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top/container_1/Reference_to_sub_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top/container_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'Sub_FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:566e5d9f-f6bc-468a-9fd5-ded0f4a10c51');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top/container_1');
catch
end
handles = get_param('PA_Root_Physical_Function/Top/Reference_to_leaf_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top/container_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:9d5f8800-df02-4681-86c0-5680b7ebd444');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top/container_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top/Reference_to_Leaf_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:8f69952d-36e4-46ae-bef9-d5dcd8138483');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top');
catch
end
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_2');
set_param('PA_Root_Physical_Function/Top_2', 'Tag', 'ModelerId:0278fdbd-9082-4bc2-bc59-3ba753307413');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_2');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_2/Reference_to_leaf_1');
set_param('PA_Root_Physical_Function/Top_2/Reference_to_leaf_1', 'Tag', 'ModelerId:2df54e0f-10fc-4498-a73d-5ef1a75ee70a');
set_param('PA_Root_Physical_Function/Top_2/Reference_to_leaf_1', 'ModelFile', 'PA_leaf_1')
handles = get_param('PA_Root_Physical_Function/Top_2/Reference_to_leaf_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:29dd7f04-b10f-41f0-8bd5-cb04a7d20f17');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_2/Reference_to_leaf_2');
set_param('PA_Root_Physical_Function/Top_2/Reference_to_leaf_2', 'Tag', 'ModelerId:178afa0b-0f08-41d8-ac7f-9db851bdf4bf');
set_param('PA_Root_Physical_Function/Top_2/Reference_to_leaf_2', 'ModelFile', 'PA_leaf_2')
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_2/container_1');
set_param('PA_Root_Physical_Function/Top_2/container_1', 'Tag', 'ModelerId:9df477a4-df2e-45ca-b5b7-e3d8beb58477');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_2/container_1');
add_block('simulink/Ports & Subsystems/In1', 'PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_1');
set_param('PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_1', 'Tag', 'ModelerId:260b2635-00bc-43f8-a1b1-12f639e39480');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_2');
set_param('PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_2', 'Port', '1');
set_param('PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_2', 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_2/container_1/Reference_to_sub_1');
set_param('PA_Root_Physical_Function/Top_2/container_1/Reference_to_sub_1', 'Tag', 'ModelerId:003b2df1-c09c-48e7-9811-5b9563057a96');
set_param('PA_Root_Physical_Function/Top_2/container_1/Reference_to_sub_1', 'ModelFile', 'PA_sub_1')
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/Reference_to_sub_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:b4ad6c0a-7966-420f-bb2f-696a36b99c08');
clear handles;
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_2/container_1/sub_2');
set_param('PA_Root_Physical_Function/Top_2/container_1/sub_2', 'Tag', 'ModelerId:2b467853-ab2c-436a-8cf3-fb59304ae4c0');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_2/container_1/sub_2');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Root_Physical_Function/Top_2/container_1/sub_2/FunctionalExchange_2');
set_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/FunctionalExchange_2', 'Port', '1');
set_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/FunctionalExchange_2', 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
add_block('simulink/Ports & Subsystems/In1', 'PA_Root_Physical_Function/Top_2/container_1/sub_2/sub_FunctionalExchange_1');
set_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/sub_FunctionalExchange_1', 'Port', '1');
set_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/sub_FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_sub_FunctionalExchange_1');
set_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/sub_FunctionalExchange_1', 'Tag', 'ModelerId:92dfd0ca-0b71-4235-b595-9c326ec57e82');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_2/container_1/sub_2/Reference_to_bottom_1');
set_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/Reference_to_bottom_1', 'Tag', 'ModelerId:18d24771-6019-4037-bbd9-8be358085c80');
set_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/Reference_to_bottom_1', 'ModelFile', 'PA_bottom_1')
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/Reference_to_bottom_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
clear handles;
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/Reference_to_bottom_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/FunctionalExchange_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_2/container_1/sub_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/sub_FunctionalExchange_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/sub_2/Reference_to_bottom_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_2/container_1/sub_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sub_FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:92dfd0ca-0b71-4235-b595-9c326ec57e82');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_2/container_1/sub_2');
catch
end
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/Reference_to_sub_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_2/container_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:260b2635-00bc-43f8-a1b1-12f639e39480');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/sub_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/FunctionalExchange_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_2/container_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/Reference_to_sub_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_2/container_1/sub_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_2/container_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sub_FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:65a7b34a-9a23-47f3-81b7-d22c644c694d');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_2/container_1');
catch
end
handles = get_param('PA_Root_Physical_Function/Top_2/Reference_to_leaf_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_2/container_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:be094c00-f0b3-440a-a0c2-1e7febcd20df');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_2/container_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_2/Reference_to_leaf_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:8aaa3039-dc85-46ef-a201-4d26e7873993');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_2');
catch
end
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_3');
set_param('PA_Root_Physical_Function/Top_3', 'Tag', 'ModelerId:de7ad848-8f32-4ac7-b325-c9661dca35d4');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_3');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_3/Reference_to_leaf1');
set_param('PA_Root_Physical_Function/Top_3/Reference_to_leaf1', 'Tag', 'ModelerId:1e5ed3de-ecc9-4902-ba70-71eeb58c021b');
set_param('PA_Root_Physical_Function/Top_3/Reference_to_leaf1', 'ModelFile', 'PA_leaf1')
handles = get_param('PA_Root_Physical_Function/Top_3/Reference_to_leaf1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:9cb0754d-bd10-4738-b67e-21590601f767');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_3/Reference_to_leaf2');
set_param('PA_Root_Physical_Function/Top_3/Reference_to_leaf2', 'Tag', 'ModelerId:ee115b68-09f3-482f-9584-282bab1f6864');
set_param('PA_Root_Physical_Function/Top_3/Reference_to_leaf2', 'ModelFile', 'PA_leaf2')
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_3/container');
set_param('PA_Root_Physical_Function/Top_3/container', 'Tag', 'ModelerId:2485b2d7-b4db-4990-8e77-846d93b1b030');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_3/container');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Root_Physical_Function/Top_3/container/bottom2_to_leaf2');
set_param('PA_Root_Physical_Function/Top_3/container/bottom2_to_leaf2', 'Port', '1');
set_param('PA_Root_Physical_Function/Top_3/container/bottom2_to_leaf2', 'OutDataTypeStr', 'Bus: busFE_bottom2_to_leaf2');
set_param('PA_Root_Physical_Function/Top_3/container/bottom2_to_leaf2', 'BusOutputAsStruct', 'on');
set_param('PA_Root_Physical_Function/Top_3/container/bottom2_to_leaf2', 'Tag', 'ModelerId:9de1a4fc-6543-4647-8fd3-60098c5dabe0');
add_block('simulink/Ports & Subsystems/In1', 'PA_Root_Physical_Function/Top_3/container/leaf1_to_bottom1');
set_param('PA_Root_Physical_Function/Top_3/container/leaf1_to_bottom1', 'Port', '1');
set_param('PA_Root_Physical_Function/Top_3/container/leaf1_to_bottom1', 'OutDataTypeStr', 'Bus: busFE_leaf1_to_bottom1');
set_param('PA_Root_Physical_Function/Top_3/container/leaf1_to_bottom1', 'Tag', 'ModelerId:1c92e414-4dc0-445a-9cfa-419fa214911f');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_3/container/Reference_to_bottom2');
set_param('PA_Root_Physical_Function/Top_3/container/Reference_to_bottom2', 'Tag', 'ModelerId:ffa504fd-8e29-4095-8e15-dd41f7e08583');
set_param('PA_Root_Physical_Function/Top_3/container/Reference_to_bottom2', 'ModelFile', 'PA_bottom2')
handles = get_param('PA_Root_Physical_Function/Top_3/container/Reference_to_bottom2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:9de1a4fc-6543-4647-8fd3-60098c5dabe0');
clear handles;
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_3/container/sub');
set_param('PA_Root_Physical_Function/Top_3/container/sub', 'Tag', 'ModelerId:1173662b-cfe4-4a2f-9e14-ec1b4d3d7193');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_3/container/sub');
add_block('simulink/Ports & Subsystems/In1', 'PA_Root_Physical_Function/Top_3/container/sub/leaf1_to_bottom1');
set_param('PA_Root_Physical_Function/Top_3/container/sub/leaf1_to_bottom1', 'Port', '1');
set_param('PA_Root_Physical_Function/Top_3/container/sub/leaf1_to_bottom1', 'OutDataTypeStr', 'Bus: busFE_leaf1_to_bottom1');
set_param('PA_Root_Physical_Function/Top_3/container/sub/leaf1_to_bottom1', 'Tag', 'ModelerId:1c92e414-4dc0-445a-9cfa-419fa214911f');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Root_Physical_Function/Top_3/container/sub/sub');
set_param('PA_Root_Physical_Function/Top_3/container/sub/sub', 'Port', '1');
set_param('PA_Root_Physical_Function/Top_3/container/sub/sub', 'OutDataTypeStr', 'Bus: busFE_sub');
set_param('PA_Root_Physical_Function/Top_3/container/sub/sub', 'BusOutputAsStruct', 'on');
set_param('PA_Root_Physical_Function/Top_3/container/sub/sub', 'Tag', 'ModelerId:395722eb-7184-475d-81dc-21e78f051c2b');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_3/container/sub/Reference_to_bottom1');
set_param('PA_Root_Physical_Function/Top_3/container/sub/Reference_to_bottom1', 'Tag', 'ModelerId:0d18653e-f4cb-4b57-9d99-6fa587a4634f');
set_param('PA_Root_Physical_Function/Top_3/container/sub/Reference_to_bottom1', 'ModelFile', 'PA_bottom1')
handles = get_param('PA_Root_Physical_Function/Top_3/container/sub/Reference_to_bottom1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:395722eb-7184-475d-81dc-21e78f051c2b');
clear handles;
handles = get_param('PA_Root_Physical_Function/Top_3/container/sub/leaf1_to_bottom1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_3/container/sub/Reference_to_bottom1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_3/container/sub', source, target, 'autorouting','smart');
set_param(line, 'Name', 'leaf1_to_bottom1');
set_param( line, 'Tag', 'ModelerId:1c92e414-4dc0-445a-9cfa-419fa214911f');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_3/container/sub/Reference_to_bottom1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_3/container/sub/sub', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_3/container/sub', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sub');
set_param( line, 'Tag', 'ModelerId:395722eb-7184-475d-81dc-21e78f051c2b');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_3/container/sub');
catch
end
handles = get_param('PA_Root_Physical_Function/Top_3/container/Reference_to_bottom2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_3/container/bottom2_to_leaf2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_3/container', source, target, 'autorouting','smart');
set_param(line, 'Name', 'bottom2_to_leaf2');
set_param( line, 'Tag', 'ModelerId:9de1a4fc-6543-4647-8fd3-60098c5dabe0');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_3/container/leaf1_to_bottom1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_3/container/sub', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_3/container', source, target, 'autorouting','smart');
set_param(line, 'Name', 'leaf1_to_bottom1');
set_param( line, 'Tag', 'ModelerId:1c92e414-4dc0-445a-9cfa-419fa214911f');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_3/container/sub', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_3/container/Reference_to_bottom2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_3/container', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sub');
set_param( line, 'Tag', 'ModelerId:a06d23f6-6296-4637-93b3-1780ead25ffc');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_3/container');
catch
end
handles = get_param('PA_Root_Physical_Function/Top_3/container', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_3/Reference_to_leaf2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_3', source, target, 'autorouting','smart');
set_param(line, 'Name', 'bottom2_to_leaf2');
set_param( line, 'Tag', 'ModelerId:48620e46-8811-4368-99a4-4d6401053a1e');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_3/Reference_to_leaf1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_3/container', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_3', source, target, 'autorouting','smart');
set_param(line, 'Name', 'leaf1_to_bottom1');
set_param( line, 'Tag', 'ModelerId:f29a19c0-7886-4b9b-9ff0-df0bb36dbcb8');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_3');
catch
end
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_4');
set_param('PA_Root_Physical_Function/Top_4', 'Tag', 'ModelerId:8d0e5a18-f1b6-4f54-9d2c-c9e96e1bd4a5');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_4');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_4/Reference_to_Leaf_4');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_4', 'Tag', 'ModelerId:b873a449-7c4e-4427-a059-a90dd4315e30');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_4', 'ModelFile', 'PA_Leaf_4')
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_4/Reference_to_Container_1');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_1', 'Tag', 'ModelerId:385587e8-9c47-4d4c-9576-f35641c14163');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_1', 'ModelFile', 'PA_Container_1')
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:fc925960-f543-4069-9182-7b2f4c2e0351');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_4/Reference_to_Leaf_1');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_1', 'Tag', 'ModelerId:5afe0c00-0313-479a-976f-b7771b500c94');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_1', 'ModelFile', 'PA_Leaf_1')
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:24869e7d-82a3-4486-89e0-d4b7ab52ca54');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_4/Reference_to_Leaf_2');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_2', 'Tag', 'ModelerId:b25c4b83-b531-41ff-87f8-a8b686dba851');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_2', 'ModelFile', 'PA_Leaf_2')
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_4/Reference_to_Leaf_3');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_3', 'Tag', 'ModelerId:56c1f50d-ebad-4103-968f-25baf7557a2b');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_3', 'ModelFile', 'PA_Leaf_3')
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_3', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:225d55f5-aa29-4389-97e4-feb621c8556f');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_4/Reference_to_Container_2');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_2', 'Tag', 'ModelerId:e7b08005-c709-4963-9826-d6188a34887f');
set_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_2', 'ModelFile', 'PA_Container_2')
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:b71f74d4-36d7-4987-853a-dd60711e53f0');
clear handles;
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_2', 'porthandles');
set_param(handles.Outport(2), 'Tag', 'ModelerId:92b2407a-330e-4007-97d7-60dd613d4f2e');
clear handles;
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_2', 'PortHandles');
source = handles.Outport(2);
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_4', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_4');
set_param( line, 'Tag', 'ModelerId:4020f037-79ae-4ffb-be04-9ce7acc43a2e');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:40af6851-833b-46fc-add0-95e6d433ab9f');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_3', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_1', 'PortHandles');
target = handles.Inport(2);
line = add_line('PA_Root_Physical_Function/Top_4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_5');
set_param( line, 'Tag', 'ModelerId:4e8e5e1e-e889-4221-8c1a-3af14c740db5');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:632b6ef3-c35a-41d2-997f-f80170fff7d4');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Container_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_4/Reference_to_Leaf_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_3');
set_param( line, 'Tag', 'ModelerId:46b3c1db-b44d-4667-bc1f-6a8d3b60fab2');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_4');
catch
end
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_5');
set_param('PA_Root_Physical_Function/Top_5', 'Tag', 'ModelerId:8748fc96-da5d-44b6-b3da-9b442c328df3');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_5');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_5/Reference_to_leaf1');
set_param('PA_Root_Physical_Function/Top_5/Reference_to_leaf1', 'Tag', 'ModelerId:81c8a58d-1798-4581-8e63-73e9b5e67f35');
set_param('PA_Root_Physical_Function/Top_5/Reference_to_leaf1', 'ModelFile', 'PA_leaf1')
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_5/container');
set_param('PA_Root_Physical_Function/Top_5/container', 'Tag', 'ModelerId:1798c7cb-3d42-48a2-be9b-2d16ef9c9b24');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_5/container');
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_5/container/container1');
set_param('PA_Root_Physical_Function/Top_5/container/container1', 'Tag', 'ModelerId:5d08f6cc-97e2-426e-b4c7-368d15959c1f');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_5/container/container1');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_5/container/container1/Reference_to_leaf2');
set_param('PA_Root_Physical_Function/Top_5/container/container1/Reference_to_leaf2', 'Tag', 'ModelerId:82c22b93-0695-45f9-a80e-07244eaced6b');
set_param('PA_Root_Physical_Function/Top_5/container/container1/Reference_to_leaf2', 'ModelFile', 'PA_leaf2')
handles = get_param('PA_Root_Physical_Function/Top_5/container/container1/Reference_to_leaf2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:74707e01-4261-4020-ad5e-b0cefc568ed0');
clear handles;
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_5/container/container1/container2');
set_param('PA_Root_Physical_Function/Top_5/container/container1/container2', 'Tag', 'ModelerId:e4c9f262-55d2-4446-b865-ba7990ec8516');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_5/container/container1/container2');
add_block('simulink/Ports & Subsystems/In1', 'PA_Root_Physical_Function/Top_5/container/container1/container2/FunctionalExchange_1');
set_param('PA_Root_Physical_Function/Top_5/container/container1/container2/FunctionalExchange_1', 'Port', '1');
set_param('PA_Root_Physical_Function/Top_5/container/container1/container2/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_Root_Physical_Function/Top_5/container/container1/container2/FunctionalExchange_1', 'Tag', 'ModelerId:2d99ce12-ffe0-4139-b3c6-63bbda0978ab');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_5/container/container1/container2/Reference_to_leaf3');
set_param('PA_Root_Physical_Function/Top_5/container/container1/container2/Reference_to_leaf3', 'Tag', 'ModelerId:03f7e903-d08f-4375-8c0f-964f1ed50ec5');
set_param('PA_Root_Physical_Function/Top_5/container/container1/container2/Reference_to_leaf3', 'ModelFile', 'PA_leaf3')
handles = get_param('PA_Root_Physical_Function/Top_5/container/container1/container2/FunctionalExchange_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_5/container/container1/container2/Reference_to_leaf3', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_5/container/container1/container2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:2d99ce12-ffe0-4139-b3c6-63bbda0978ab');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_5/container/container1/container2');
catch
end
handles = get_param('PA_Root_Physical_Function/Top_5/container/container1/Reference_to_leaf2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_5/container/container1/container2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_5/container/container1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:8e52be06-ff6f-4fb2-b496-571d8ac36692');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_5/container/container1');
catch
end
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_5/container');
catch
end
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_5');
catch
end
add_block('CapellaElements/Subsystem/Function', 'PA_Root_Physical_Function/Top_6');
set_param('PA_Root_Physical_Function/Top_6', 'Tag', 'ModelerId:26272849-2349-45f9-9e8b-80b131e5d506');
CapSLBridge.internal.cleanSubsystem('PA_Root_Physical_Function/Top_6');
add_block('simulink/Ports & Subsystems/In1', 'PA_Root_Physical_Function/Top_6/EN_pf_1');
set_param('PA_Root_Physical_Function/Top_6/EN_pf_1', 'Port', '1');
add_block('simulink/Ports & Subsystems/In1', 'PA_Root_Physical_Function/Top_6/EN_pf_2');
set_param('PA_Root_Physical_Function/Top_6/EN_pf_2', 'Port', '2');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_6/Reference_to_pf_1');
set_param('PA_Root_Physical_Function/Top_6/Reference_to_pf_1', 'Tag', 'ModelerId:36ece6e6-ae73-47b2-b9b3-9e102f6cee51');
set_param('PA_Root_Physical_Function/Top_6/Reference_to_pf_1', 'ModelFile', 'PA_pf_1')
handles = get_param('PA_Root_Physical_Function/Top_6/Reference_to_pf_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:d12d6e5e-326b-40f5-9dbf-8b98a7511d75');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Root_Physical_Function/Top_6/Reference_to_pf_2');
set_param('PA_Root_Physical_Function/Top_6/Reference_to_pf_2', 'Tag', 'ModelerId:093e62cb-9bb3-4141-8507-4229af11d8e0');
set_param('PA_Root_Physical_Function/Top_6/Reference_to_pf_2', 'ModelFile', 'PA_pf_2')
handles = get_param('PA_Root_Physical_Function/Top_6/EN_pf_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_6/Reference_to_pf_1', 'PortHandles');
target = handles.Enable(1);
line = add_line('PA_Root_Physical_Function/Top_6', source, target, 'autorouting','smart');
set_param(line, 'Name', 'EN_pf_1');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_6/EN_pf_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_6/Reference_to_pf_2', 'PortHandles');
target = handles.Enable(1);
line = add_line('PA_Root_Physical_Function/Top_6', source, target, 'autorouting','smart');
set_param(line, 'Name', 'EN_pf_2');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/Top_6/Reference_to_pf_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_6/Reference_to_pf_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function/Top_6', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:6e911035-42a1-434b-b1c3-f9c696de00e8');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function/Top_6');
catch
end
handles = get_param('PA_Root_Physical_Function/EN_pf_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_6', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Root_Physical_Function', source, target, 'autorouting','smart');
set_param(line, 'Name', 'EN_pf_1');
clear line handles source target;
handles = get_param('PA_Root_Physical_Function/EN_pf_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Root_Physical_Function/Top_6', 'PortHandles');
target = handles.Inport(2);
line = add_line('PA_Root_Physical_Function', source, target, 'autorouting','smart');
set_param(line, 'Name', 'EN_pf_2');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Root_Physical_Function');
catch
end
save_system('PA_Root_Physical_Function', fullfile(project_path, 'models/PA/PA_Root_Physical_Function.slx'));
file = proj.addFile('models/PA/PA_Root_Physical_Function.slx');
addLabel(file, 'Capella Layer', 'PA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
