
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
            var contactStore = await ContactManager.RequestStoreAsync();
            return (await contactStore.FindContactsAsync()).Select(c => ContactToPerson(c)).Where(c => c != null).ToList();
        }

        Person ContactToPerson(Contact contact)
        {
            try
            {
                string id = contact.Id;
                string firstName = contact.FirstName;
                string lastName = contact.LastName;
                DateTimeOffset date = contact.ImportantDates.FirstOrDefault(d => d.Kind == ContactDateKind.Birthday).ToDateTimeOffset();
                string phoneNumber = contact.Phones.FirstOrDefault()?.Number;
                string address = contact.Addresses.FirstOrDefault().ToFullAddress();
                Guardian guardian = Guardian.FromText(contact.Notes);

                return new Person(id, firstName, lastName, date, phoneNumber, address, guardian);
            }
            catch
            {
                return null;
                //TODO report
            }
        }

        public override async Task AddPerson(Person person)
        {
            try
            {
                var contact = PersonToContact(person);
                await SaveContact(contact);
            }
            catch
            {
                //TODO handle error
            }
        }

        Contact PersonToContact(Person person)
        {
            var contact = new Contact
            {
                Id = person.Id,
                FirstName = person.FirstName,
                LastName = person.LastName,
                Notes = person.Guardian?.ToString()
            };
            contact.Phones.Add(new ContactPhone() { Kind = ContactPhoneKind.Mobile, Number = person.PhoneNumber });
            contact.Addresses.Add(person.Address.ToContactAddress());
            if (person.BirthDate != new DateTimeOffset())
                contact.ImportantDates.Add(person.BirthDate.ToContactDate());

            return contact;
        }

        async Task SaveContact(Contact contact)
        {
            var contactStore = await ContactManager.RequestStoreAsync(ContactStoreAccessType.AppContactsReadWrite);

            var contactList = (await contactStore.FindContactListsAsync()).FirstOrDefault();
            if (contactList == null)
                throw new NullReferenceException("Could not find a contact list to add the contact to.");

            await contactList.SaveContactAsync(contact);
        }
    }

    static class ContactExtensions
    {
        #region Dates

        public static DateTimeOffset ToDateTimeOffset(this ContactDate contactDate)
        {
            if (contactDate == null || contactDate.Year == null || contactDate.Month == null || contactDate.Day == null)
                return default(DateTimeOffset);

            return new DateTimeOffset(new DateTime(contactDate.Year.Value, Convert.ToInt32(contactDate.Month.Value), Convert.ToInt32(contactDate.Day.Value)));
        }

        public static ContactDate ToContactDate(this DateTimeOffset dateTimeOffset)
        {
            return new ContactDate()
            {
                Year = dateTimeOffset.Year,
                Month = Convert.ToUInt32(dateTimeOffset.Month),
                Day = Convert.ToUInt32(dateTimeOffset.Day)
            };
        }

        #endregion

        #region Addresses

        public static string ToFullAddress(this ContactAddress address)
        {
            if (address == null)
                return null;

            return address.ToString(); //TODO
        }

        public static ContactAddress ToContactAddress(this string fullAddress)
        {
            return new ContactAddress(); //TODO
        }

        #endregion
    }
}
