
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Windows.ApplicationModel.Contacts;
using Windows.Storage;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn
{
    public class UwpDataProvider : DataProvider
    {
        private const string birthDateName = "Birth date";

        protected override async Task<string> GetJsonFileContent(string fileName)
        {
            var rootFolder = await GetRootFolder();
            var item = await rootFolder.TryGetItemAsync(fileName);

            if (item is IStorageFile file)
                return await FileIO.ReadTextAsync(file);
            else
                return null;
        }

        private static async Task<StorageFolder> GetRootFolder() =>
            await KnownFolders.DocumentsLibrary.CreateFolderAsync("Youth Center Sign In", CreationCollisionOption.OpenIfExists);

        protected override string GetJsonSetting(string key)
        {
            return (string)ApplicationData.Current.LocalSettings.Values[key];
        }

        protected override async Task SetJsonFileContent(string fileName, string json)
        {
            var rootFolder = await GetRootFolder();
            var file = await rootFolder.CreateFileAsync(fileName, CreationCollisionOption.OpenIfExists);

            await FileIO.WriteTextAsync(file, json);
        }

        protected override void SetJsonSetting(string key, string json)
        {
            ApplicationData.Current.LocalSettings.Values[key] = json;
        }

        public override async Task<List<Person>> GetPeople()
        {
            Person ContactToPerson(Contact contact)
            {
                return new Person(contact.Id, contact.FirstName, contact.LastName);
            }

            var contactStore = await ContactManager.RequestStoreAsync();
            return (await contactStore.FindContactsAsync()).Select(c => ContactToPerson(c)).ToList();
        }

        public override async Task AddPerson(Person person)
        {
            var contact = PersonToContact(person);
            await SaveContact(contact);
        }

        Contact PersonToContact(Person person)
        {
            return new Contact
            {
                FirstName = person.FirstName,
                LastName = person.LastName,
                //TODO
            };
        }

        async Task SaveContact(Contact contact)
        {
            try
            {
                var contactStore = await ContactManager.RequestStoreAsync(ContactStoreAccessType.AppContactsReadWrite);

                var contactList = (await contactStore.FindContactListsAsync()).FirstOrDefault();
                if (contactList == null)
                    throw new NullReferenceException("Could not find a contact list to add the contact to.");

                await contactList.SaveContactAsync(contact);
            }
            catch
            {
                //TODO handle error
            }
        }
    }
}
